#include <atomic>
#include <memory>
#include <thread>
#include <vector>
#include <cstdio>

struct S {
    S(int n) : a{n} {};
    void increment() { a++; };

    int a;
};

void worker(std::shared_ptr<S> variable) {
	variable->increment();
}

int main()
{
	std::vector<std::thread> threads;
	std::shared_ptr<S> variable = std::make_shared<S>(0);

	for (int i = 0; i < 32; i++) {
		std::thread t(worker, variable);
		threads.emplace_back(std::move(t));
	}

	for (std::thread &t : threads) {
		t.join();
	}

	std::printf("Total: %d\n", variable->a);

	return 0;
}
