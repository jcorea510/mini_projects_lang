#include <serialib.h>
#include <print>
#include <chrono>
#include <random>


int main() {
    char chars[] = {0x01, 0x02, 0x03, 0x04};
    int size = sizeof(chars) / sizeof(chars[0]);

    // Set up a random number generator
    std::random_device rd;
    std::mt19937 gen(rd());
    std::uniform_int_distribution<> distr(0, size - 1);

    std::string serial_port  = "/dev/ttyUSB0"; // Blue Pill USB-CDC default
    unsigned int serial_baud = 9600;

    serialib serial;
	if (serial.openDevice(serial_port.c_str(), serial_baud) != 1) {
		std::cerr << "[Serial] Init failed, continuing without serial.\n";
		return -1;
    }

	
	auto last_send = std::chrono::steady_clock::now();
	while (1) {
		auto now = std::chrono::steady_clock::now();
		if (now - last_send > std::chrono::milliseconds(200)) {
			char randomchar = chars[distr(gen)];
			serial.writeChar(randomchar);
			std::println("Sended gesture: {:02x}", randomchar);
			last_send = now;
		}

		// --- Read echo (non-blocking, 10ms timeout) ---
        char echo = 0;
        int ret = serial.readChar(&echo, 10);
        if (ret == 1) {
            std::println("RX echo: 0x{:02x} '{}'",
                static_cast<unsigned char>(echo),
                (echo >= 0x20 && echo < 0x7f) ? echo : '?');
        } else if (ret == 0) {
            // timeout, nothing received yet — normal
        } else {
            std::println("RX error: {}", ret);
        }
	}

	return 0;
}
