#include <components.hpp>
#include <simulator.hpp>
#include <plotter.hpp>
#include <Eigen/Dense>
#include <vector>
#include <netlist.hpp>
#include <filesystem>
#include <argparse/argparse.hpp>

int main(int argc, char *argv[]) {
	// argument parser
	argparse::ArgumentParser program("circuitx");
	program.add_argument("netlist")
		.help("netlist to analyze");


	try {
		program.parse_args(argc, argv);
	}
	catch (const std::exception& err) {
		std::cerr << err.what() << std::endl;
		std::cerr << program;
		std::exit(1);
	}

	auto input = program.get<std::string>("netlist");
	if (input.empty()) {
		return -1;
	}
	
	// netlist reading
	std::filesystem::path filename(input);
	NetList netlist = readNetlist(filename);

	MNA mna = defineMNA(netlist);

	// AC trasient with euler
	SimConf simconfig = readSimConfig(filename);
	std::vector<Eigen::VectorXd> wave = simulate_backward_euler(mna, netlist, simconfig);
	
	// save results
	std::filesystem::path path_to_save = filename.parent_path();
	time_ploter(wave, netlist, simconfig, path_to_save);
    return 0;
}
