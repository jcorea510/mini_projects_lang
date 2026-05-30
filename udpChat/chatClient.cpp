#include <arpa/inet.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <netinet/in.h>
#include <print>
#include <string>
#include <sys/socket.h>
#include <thread>

// Runs in background, prints messages forwarded by the server
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
		char* space_ptr = std::strchr(buffer, ' ');
		if (space_ptr != nullptr) {
			// Null-terminate the first part to split the buffer
			*space_ptr = '\0'; 
			// The second part starts right after the original space
			char* sender_port = buffer;
			char* message = space_ptr + 1;

			std::println("[Incoming] {} says: {}", sender_port, message);
		}
	}
}

int main(int argc, char* argv[]) {
	if (argc != 2) {
		std::println("Usage: {} <local-port>", argv[0]);
		exit(EXIT_FAILURE);
	}

	int local_port = std::atoi(argv[1]);
	std::println("Chat client");

	int sockfd;
	if ((sockfd = socket(AF_INET, SOCK_DGRAM, 0)) < 0) {
		perror("Socket creating failed");
		exit(EXIT_FAILURE);
	}

	// bind to local_port so the server registers the port we intend
	sockaddr_in local_addr{};
	memset(&local_addr, 0, sizeof(local_addr));
	local_addr.sin_family      = AF_INET;
	local_addr.sin_addr.s_addr = INADDR_ANY;
	local_addr.sin_port        = htons(local_port);

	if ((bind(sockfd, (const struct sockaddr *)&local_addr, sizeof(local_addr))) < 0) {
		perror("Bind failed");
		exit(EXIT_FAILURE);
	}

	sockaddr_in server_addr{};
	memset(&server_addr, 0, sizeof(server_addr));
	server_addr.sin_family      = AF_INET;
	server_addr.sin_addr.s_addr = inet_addr("127.0.0.1");
	server_addr.sin_port        = htons(8080);
	socklen_t server_len = sizeof(server_addr);

	// Register with the server
	std::string user_id;
	std::print("Enter your user ID: ");
	if (!std::getline(std::cin, user_id) || user_id.empty()) {
		std::println("Invalid user ID");
		exit(EXIT_FAILURE);
	}

	std::string reg_cmd = "REGISTER " + user_id;
	int n = sendto(sockfd, reg_cmd.c_str(), reg_cmd.length(),
			MSG_CONFIRM, (const struct sockaddr *)&server_addr, server_len);
	if (n < 0) {
		perror("sendto");
		exit(EXIT_FAILURE);
	}
	std::println("Registered as '{}'", user_id);

	// ask who to talk to
	std::string target;
	std::print("Send messages to: ");
	if (!std::getline(std::cin, target) || target.empty()) {
		std::println("Invalid target");
		exit(EXIT_FAILURE);
	}

	// start receive thread so we can get messages while sending
	std::thread t(receive_thread, sockfd);
	t.detach();

	// Send loop
	std::string message;
	while (true) {
		if (!std::getline(std::cin, message)) break;
		if (message.empty()) continue;

		std::string cmd = "SEND " + target + " " + message;
		int n = sendto(sockfd, cmd.c_str(), cmd.length(),
				MSG_CONFIRM, (const struct sockaddr *)&server_addr, server_len);
		if (n < 0) {
			perror("sendto");
			break;
		}
	}

	return 0;
}
