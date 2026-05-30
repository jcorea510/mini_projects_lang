#include <arpa/inet.h>
#include <condition_variable>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <netinet/in.h>
#include <print>
#include <queue>
#include <string>
#include <sys/socket.h>
#include <map>
#include <mutex>
#include <thread>
#include <utility>

std::condition_variable cv;
std::mutex m;

// shared resources
std::map<std::string, int> id_port;
std::queue<std::pair<int, std::string>> port_message;

// Receives packets from clients, parses commands, and queues forwarding jobs
void receive_thread(int fd) {
	char buffer[1024];
	sockaddr_in sender_addr{};
	socklen_t len = sizeof(sender_addr);

	while (true) {
		int n = recvfrom(fd, buffer, sizeof(buffer) - 1, MSG_WAITALL,
					 (struct sockaddr *)&sender_addr, &len);
		if (n < 0) {
			perror("recvfrom");
			break;
		}
		buffer[n] = '\0';

		char sender_ip[INET_ADDRSTRLEN];
		inet_ntop(AF_INET, &sender_addr.sin_addr, sender_ip, sizeof(sender_ip));
		int sender_port = ntohs(sender_addr.sin_port);

		std::string command(buffer);

		if (command.starts_with("REGISTER ")) {
			std::string user_id = command.substr(9);
			std::lock_guard<std::mutex> lock(m);
			id_port[user_id] = sender_port;
			std::println("[Server] Registered '{}' on port {}", user_id, sender_port);
		}
		else if (command.starts_with("SEND ")) {
			// use find() to split on the space delimiter, no fixed-width assumption
			std::string rest = command.substr(5);
			auto space = rest.find(' ');
			if (space == std::string::npos) {
				std::println("[Server] Malformed SEND (missing target/message separator)");
				continue;
			}
			std::string target  = rest.substr(0, space);

			std::string sender_port_str;
			for (auto const [key, value]: id_port) {
				if (value == sender_port) {
					sender_port_str = key + " ";	
				}
			}
			std::string message = sender_port_str + rest.substr(space + 1);

			std::lock_guard<std::mutex> lock(m);
			if (id_port.contains(target)) {
				// notify forward_thread there is work to do
				port_message.push({id_port[target], message});
				cv.notify_one();
			} else {
				std::println("[Server] Unknown target '{}'", target);
			}
		}
	}
}

// Waits for queued messages and forwards them to the right client port
void forward_thread(int fd) {
	while (true) {
		std::unique_lock<std::mutex> lock(m);
		cv.wait(lock, [] { return !port_message.empty(); });

		while (!port_message.empty()) {
			auto [port, message] = port_message.front();
			port_message.pop();
			lock.unlock(); // unlock while doing I/O so receive_thread isn't blocked

			sockaddr_in target_addr{};
			memset(&target_addr, 0, sizeof(target_addr));
			target_addr.sin_family      = AF_INET;
			target_addr.sin_addr.s_addr = inet_addr("127.0.0.1");
			target_addr.sin_port        = htons(port);

			int n = sendto(fd, message.c_str(), message.length(),
					MSG_CONFIRM, (const struct sockaddr *)&target_addr, sizeof(target_addr));
			if (n < 0) perror("sendto");

			lock.lock();
		}
	}
}

int main() {
	std::println("Chat server starting on port 8080...");

	int sockfd;
	if ((sockfd = socket(AF_INET, SOCK_DGRAM, 0)) < 0) {
		perror("Socket creating failed");
		exit(EXIT_FAILURE);
	}

	sockaddr_in server_addr{};
	memset(&server_addr, 0, sizeof(server_addr));
	server_addr.sin_family      = AF_INET;
	server_addr.sin_addr.s_addr = INADDR_ANY;
	server_addr.sin_port        = htons(8080);

	if ((bind(sockfd, (const struct sockaddr *)&server_addr, sizeof(server_addr))) < 0) {
		perror("Bind failed");
		exit(EXIT_FAILURE);
	}

	std::thread t1(receive_thread, sockfd);
	std::thread t2(forward_thread, sockfd);

	t1.join();
	t2.join();

	return 0;
}
