#include <arpa/inet.h>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <netinet/in.h>
#include <print>
#include <string>
#include <sys/socket.h>
#include <thread>

void server_process(int fd) {
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
        std::println("[Server] {}:{} says: {}", sender_ip, ntohs(sender_addr.sin_port), buffer);
	}
}

void client_process(int fd, sockaddr_in server_addr) {
	char buffer[1024];
	std::string message;
	socklen_t len = sizeof(server_addr);

	while (true) {
		if (!std::getline(std::cin, message)) break;;
		int n = sendto(fd, message.c_str(), message.length(), MSG_CONFIRM,
					   (const struct sockaddr *)&server_addr, len);
		if (n < 0) {
			perror("sento");
			break;
		}
		std::this_thread::sleep_for(std::chrono::milliseconds(500));
	}
}

int main(int argc, char* argv[]) {
	if (argc != 3) {
		std::println("Usage: {} <loca-port> <remote-port>", argv[0]);
		return EXIT_FAILURE;
	}

	int local_port = std::stoi(argv[1]);
	int remote_port = std::stoi(argv[2]);

	int sockfd;
	if ((sockfd = socket(AF_INET, SOCK_DGRAM, 0)) < 0) {
		perror("Socket creating failed");
		exit(EXIT_FAILURE);
	}

	sockaddr_in server_addr{};
	sockaddr_in client_addr{};
	memset(&server_addr, 0, sizeof(server_addr));
	memset(&client_addr, 0, sizeof(client_addr));

	server_addr.sin_family = AF_INET;
	server_addr.sin_addr.s_addr = INADDR_ANY;
	server_addr.sin_port = htons(local_port);

	client_addr.sin_family = AF_INET;
	client_addr.sin_addr.s_addr = inet_addr("127.0.0.1");
	client_addr.sin_port = htons(remote_port);

	if ((bind(sockfd, (const struct sockaddr *)&server_addr, 
					sizeof(server_addr))) < 0) {
		perror("Bind failed");
		exit(EXIT_FAILURE);
	}

    std::println("Listening on :{}, sending to :{}", local_port, remote_port);
	std::thread server_task(server_process, sockfd);
	std::thread client_task(client_process, sockfd, client_addr);

	server_task.join();
	client_task.join();

	return 0;
}
