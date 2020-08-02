#include <atomic>
#include <chrono>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <iostream>
#include <map>
#include <memory>
#include <mutex>
#include <thread>
#include <vector>

#include <argp.h>
#include <curl/curl.h>
#include <getopt.h>
#include <sys/resource.h>

// argp stuff
const char* argp_program_version = "url-fuzzer.0.0.1";
const char* argp_program_bug_address = "<romeu.bizz@gmail.com>";
static char doc[] = "Software to do some URL Fuzzing.";
static char args_doc[] = "[URL]...";
static struct argp_option options[] = {
    {"url", 'u', "value", 0, "Url to fuzz"},
    {"wordlist", 'w', "value", 0, "Wordlist with 1 string per line"},
    {"threads", 't', "value", 0, "Number of threads"},
    {0}
};

struct ArgOpts {
    char* argu;
    char* argw;
    char* argt;
    int optu;
    int optw;
    int optt;
};

ArgOpts pargs = {};

static auto argp_parseopts(int key, char* arg, struct argp_state* state) -> error_t
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
            pargs.argt = strdup(arg);
            break;
        case ARGP_KEY_ARG:
            return 0;
        default:
            return ARGP_ERR_UNKNOWN;
    }

    return 0;
}

struct Statistics {
	std::map<std::string, std::vector<std::string>> resp_list;
	std::map<std::string, std::vector<std::string>> error_list;
};

std::vector<std::string> file_read_lines(char *file)
{
	std::ifstream in(file);

	std::string line;
	std::vector<std::string> lines;

	while (std::getline(in, line)) {
		lines.emplace_back(line);
	}

	return lines;
}

void file_write_lines(char* filename, std::vector<std::string> vec)
{
	if (std::ofstream ofs(filename); ofs) {
		for (auto &str : vec) {
			ofs << str << std::endl;
		}
	}
}

size_t write_data(void *buffer, size_t size, size_t nmemb, void *userp)
{
   return size * nmemb;
}

long request(char const* url)
{
	CURL *curl = curl_easy_init();

	curl_easy_setopt(curl, CURLOPT_URL, url);
	curl_easy_setopt(curl, CURLOPT_VERBOSE, 0L);
	curl_easy_setopt(curl, CURLOPT_CONNECTTIMEOUT, 5);
	curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_data);

	CURLcode curlcode = curl_easy_perform(curl);
	long http_code = 0;

	curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_code);

	curl_easy_cleanup(curl);

	return (curlcode == 0) ? http_code : curlcode;
}

std::mutex wordlist_mutex;
std::mutex stats_mutex;

void worker(int thread_id, std::string url, std::shared_ptr<std::vector<std::string>> wordlist, std::shared_ptr<Statistics> statistics) {

	for (;;) {
		std::string line;

		{
			std::lock_guard<std::mutex> const lock(wordlist_mutex);

			if (wordlist->size() < 1) {
				break;
			}

			line = wordlist->at(0);
			wordlist->erase(wordlist->begin());
		}

		std::string endpoint = url + line;
		long http_code = request(endpoint.c_str());

		if (http_code < 200) {
			std::printf("[%d] - %s (%s)\n", http_code, endpoint.c_str(), curl_easy_strerror((CURLcode)http_code));
		} else {
			{
				std::lock_guard<std::mutex> const lock(stats_mutex);

				auto code_as_string = std::to_string(http_code);
				statistics->resp_list[code_as_string].emplace_back(endpoint);
			}

			std::printf("[%d] - %s\n", http_code, endpoint.c_str());
	    }
	}
}

int main(int argc, char** argv)
{
	static struct argp argp = { options, argp_parseopts, args_doc, doc, 0, 0, 0 };
    argp_parse(&argp, argc, argv, 0, 0, &pargs);

	auto url = pargs.argu;
	auto file = pargs.argw;
	auto threads = std::stoi(pargs.argt);

	// TODO: check this in argp_parseopts
	if (url == nullptr) {
		std::printf("Url not valid!\n");
		exit(1);
	}

	auto wordlist = file_read_lines(file);
	if (wordlist.size() < 1) {
		std::printf("Wordlist is empty!\n");
		exit(1);
	}

	curl_global_init(CURL_GLOBAL_ALL);

	auto wordlist_shared = std::make_shared<std::vector<std::string>>(wordlist);

	std::vector<std::thread> thread_list;
	std::shared_ptr<Statistics> statistics = std::make_shared<Statistics>();

	for (int i = 0; i < threads; i++) {
		std::thread t(worker, i, url, wordlist_shared, statistics);
		thread_list.emplace_back(std::move(t));
	}

	for (std::thread &t : thread_list) {
		t.join();
	}

	curl_global_cleanup();

	struct rusage usage;
	int who = RUSAGE_SELF;
	int ret;

	ret = getrusage(who, &usage);

	std::printf("Memory usage: %d\n", usage.ru_maxrss);
	std::printf("Total file lines: %d\n", wordlist.size());
	std::printf("Stats: %d responses\n", statistics->resp_list.size());
	return 0;
}
