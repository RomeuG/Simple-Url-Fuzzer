#include <argp.h>
#include <atomic>
#include <chrono>
#include <cstdio>
#include <cstring>
#include <curl/curl.h>
#include <fstream>
#include <future>
#include <getopt.h>
#include <iostream>
#include <map>
#include <memory>
#include <mutex>
#include <signal.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <thread>
#include <uriparser/Uri.h>
#include <vector>

#define RESET "\033[0m"
#define BLACK "\033[30m" /* Black */
#define RED "\033[31m" /* Red */
#define GREEN "\033[32m" /* Green */
#define YELLOW "\033[33m" /* Yellow */
#define BLUE "\033[34m" /* Blue */
#define MAGENTA "\033[35m" /* Magenta */
#define CYAN "\033[36m" /* Cyan */
#define WHITE "\033[37m" /* White */

// argp stuff
const char* argp_program_version = "url-fuzzer.0.0.1";
const char* argp_program_bug_address = "<romeu.bizz@gmail.com>";
static char doc[] = "Software to do some URL Fuzzing.";
static char args_doc[] = "[URL]...";
static struct argp_option options[] = {
    { "url", 'u', "value", 0, "Url to fuzz" },
    { "wordlist", 'w', "value", 0, "Wordlist with 1 string per line" },
    { "threads", 't', "value", 0, "Number of threads" },
    { "timeout", 'm', "value", 0, "Timeout value" },
    { 0 }
};

std::string to_color(char* color, std::string str)
{
    return color + str + RESET;
}

std::string to_color(char* color, int n)
{
    std::string s = std::to_string(n);
    return color + s + RESET;
}

struct ArgOpts {
    char* argu;
    char* argw;
    int argt;
    int argm;

    int optu;
    int optw;
    int optt;
    int optm;
};

ArgOpts pargs = {};

static error_t argp_parseopts(int key, char* arg, struct argp_state* state)
{
    switch (key) {
        case 'u':
            pargs.optu = 1;
            pargs.argu = strdup(arg);
            break;
        case 'w':
            pargs.optw = 1;
            pargs.argw = strdup(arg);
            break;
        case 't':
            pargs.optt = 1;
            pargs.argt = std::stoi(arg);
            break;
        case 'm':
            pargs.optm = 1;
            pargs.argm = std::stoi(arg);
            break;
        case ARGP_KEY_ARG:
            return 0;
        default:
            return ARGP_ERR_UNKNOWN;
    }

    return 0;
}

void replace(std::string& str, std::string_view const from, std::string_view const to)
{
    if (from.empty()) {
        return;
    }

    std::size_t start_pos = 0;
    while ((start_pos = str.find(from, start_pos)) != std::string::npos) {
        str.replace(start_pos, from.length(), to);
        start_pos += to.length();
    }
}

static int do_mkdir(char const* path, mode_t mode)
{
    struct stat st;
    int status = 0;

    if (stat(path, &st) != 0) {
        if (mkdir(path, mode) != 0 && errno != EEXIST) {
            status = -1;
        }
    } else if (!S_ISDIR(st.st_mode)) {
        errno = ENOTDIR;
        status = -1;
    }

    return status;
}

static int mkpath(char const* path, mode_t mode)
{
    char* pp;
    char* sp;
    int status;
    char* copypath = strdup(path);

    status = 0;
    pp = copypath;
    while (status == 0 && (sp = strchr(pp, '/')) != 0) {
        if (sp != pp) {
            *sp = '\0';
            status = do_mkdir(copypath, mode);
            *sp = '/';
        }
        pp = sp + 1;
    }

    if (status == 0) {
        status = do_mkdir(path, mode);
    }

    free(copypath);

    return status;
}

std::atomic<bool> stop_threads = false;

void sigint_handler(int s)
{
    printf("Caught signal %d\n", s);
    stop_threads = true;
}

struct Statistics {
    std::map<std::string, std::vector<std::string>> resp_list;
    std::map<std::string, std::vector<std::string>> error_list;
};

std::string get_url_host(char const* url)
{
    std::string host;

    UriUriA uri;
    char const* error;

    if (uriParseSingleUriA(&uri, url, &error) != URI_SUCCESS) {
        std::printf("Failure parsing string!\n");
        exit(1);
    }

    host = std::string(uri.hostText.first);

    uriFreeUriMembersA(&uri);

    return host;
}

std::vector<std::string> file_read_lines(char const* file)
{
    std::ifstream in(file);

    std::string line;
    std::vector<std::string> lines;

    while (std::getline(in, line)) {
        lines.emplace_back(line);
    }

    return lines;
}

void file_write_lines(char const* filename, std::vector<std::string> vec)
{
    std::ofstream ofs(filename);

    if (ofs) {
        std::printf("Writing to file: %s\n", filename);
        for (auto& str : vec) {
            ofs << str << std::endl;
        }
    } else {
        std::printf("Failure opening %s for writing\n", filename);
    }
}

size_t write_data(void* buffer, size_t size, size_t nmemb, void* userp)
{
    return size * nmemb;
}

long request(char const* url)
{
    CURL* curl = curl_easy_init();

    curl_easy_setopt(curl, CURLOPT_URL, url);
    curl_easy_setopt(curl, CURLOPT_VERBOSE, 0L);
    curl_easy_setopt(curl, CURLOPT_CONNECTTIMEOUT, pargs.argm);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_data);

    CURLcode curlcode = curl_easy_perform(curl);
    long http_code = 0;

    curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_code);

    curl_easy_cleanup(curl);

    return (curlcode == 0) ? http_code : curlcode;
}

std::mutex wordlist_mutex;
std::mutex stats_mutex;

void worker(int thread_id, std::string url,
            std::shared_ptr<std::vector<std::string>> wordlist,
            std::shared_ptr<Statistics> statistics)
{
    for (;;) {
        std::string url_copy = url;

        if (stop_threads) {
            break;
        }

        std::string line;

        {
            std::lock_guard<std::mutex> const lock(wordlist_mutex);

            if (wordlist->size() < 1) {
                break;
            }

            line = wordlist->at(0);
            wordlist->erase(wordlist->begin());
        }

        replace(url_copy, "@@", line);
        long http_code = request(url_copy.c_str());

        if (http_code < 200) {
            {
                std::lock_guard<std::mutex> const lock(stats_mutex);

                auto code_as_string = std::to_string(http_code);
                statistics->error_list[code_as_string].emplace_back(url_copy);
            }

            std::printf("[%d] - %s (%s)\n", http_code, url_copy.c_str(),
                        curl_easy_strerror((CURLcode)http_code));
        } else {
            {
                std::lock_guard<std::mutex> const lock(stats_mutex);

                auto code_as_string = std::to_string(http_code);
                statistics->resp_list[code_as_string].emplace_back(url_copy);
            }

            if (http_code >= 200 && http_code < 300) {
                // std::printf("[%s] - %s\n", to_color(GREEN, http_code).c_str(), url_copy.c_str());
            } else if (http_code >= 300 && http_code < 400) {
                // std::printf("[%s] - %s\n", to_color(YELLOW, http_code).c_str(), url_copy.c_str());
            } else {
                // std::printf("[%s] - %s\n", to_color(RED, http_code).c_str(), url_copy.c_str());
            }
        }
    }
}

int main(int argc, char** argv)
{
    static struct argp argp = { options, argp_parseopts, args_doc, doc, 0, 0, 0 };
    argp_parse(&argp, argc, argv, 0, 0, &pargs);

    // TODO: check this in argp_parseopts
    if (pargs.argu == nullptr) {
        std::printf("Url not valid!\n");
        exit(1);
    }

    std::string url = pargs.argu;
    std::string file = pargs.argw;
    int threads = pargs.argt;

    if (url.find("@@") == -1) {
        std::printf("Url does not include fuzz indicator!\n");
        exit(1);
    }

    auto wordlist = file_read_lines(file.c_str());
    if (wordlist.size() < 1) {
        std::printf("Wordlist is empty!\n");
        exit(1);
    }

    curl_global_init(CURL_GLOBAL_ALL);

    // prepare signal handling
    struct sigaction sigint_action;
    sigint_action.sa_handler = sigint_handler;
    sigemptyset(&sigint_action.sa_mask);
    sigint_action.sa_flags = 0;
    sigaction(SIGINT, &sigint_action, nullptr); // quit threads and exit clean

    auto wordlist_total = wordlist.size();
    auto wordlist_shared = std::make_shared<std::vector<std::string>>(wordlist);

    std::vector<std::thread> thread_list;
    std::shared_ptr<Statistics> statistics = std::make_shared<Statistics>();

    for (int i = 0; i < threads; i++) {
        std::thread t(worker, i, url, wordlist_shared, statistics);
        thread_list.emplace_back(std::move(t));
    }

    bool threads_stopped = false;
    auto a = std::async(std::launch::async, [statistics,
                                             &wordlist_total,
                                             &wordlist_shared,
                                             &threads_stopped]() {
        while (!stop_threads && !threads_stopped) {
            auto percentage = 100.0f * (float)(wordlist_shared->size() / (float)wordlist_total);

            std::printf("(%d/%d) %0.2f Done / %d errors\n", wordlist_shared->size(), wordlist_total,
                        percentage, statistics->error_list.size());
            std::this_thread::sleep_for(std::chrono::seconds(1));
        }
    });

    for (std::thread& t : thread_list) {
        t.join();
    }

    threads_stopped = true;
    a.wait();

    curl_global_cleanup();

    std::string host = get_url_host(url.c_str());
    replace(host, "@@", "");
    mkpath(host.c_str(), 0700);

    for (auto& it : statistics->resp_list) {
        auto file_name = it.first + ".txt";
        auto path = host + file_name;

        std::printf("Begin writing to file: %s\n", path.c_str());

        file_write_lines(path.c_str(), it.second);
    }

    for (auto& it : statistics->error_list) {
        auto file_name = it.first + ".txt";
        auto path = host + file_name;

        std::printf("Begin writing to file: %s\n", path.c_str());

        file_write_lines(path.c_str(), it.second);
    }

    struct rusage usage;
    int who = RUSAGE_SELF;
    int ret;

    ret = getrusage(who, &usage);

    std::printf("Memory usage: %d\n", usage.ru_maxrss);
    std::printf("Total file lines: %d\n", wordlist.size());
    std::printf("Stats: %d responses\n", statistics->resp_list.size());

    return 0;
}
