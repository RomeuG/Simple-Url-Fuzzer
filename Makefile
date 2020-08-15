fast:
	g++ -Os -march=native -fno-exceptions -fno-rtti -mtune=native src/main.cpp -std=c++20 -lpthread -lcurl -luriparser -o suf

debug:
	g++ -O0 -ggdb3 src/main.cpp -std=c++20 -lpthread -lcurl -luriparser -o suf
