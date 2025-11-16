#include "netlist.hpp"
#include <format>
#include <plotter.hpp>
#include <matplot/matplot.h>
#include <string>
#include <filesystem>

void time_ploter(
		std::vector<Eigen::VectorXd> &wave,
		NetList &netlist,
		SimConf &simconf,
		std::filesystem::path &path_to_save)
{
	const size_t steps = wave.size();

	if (steps == 0) { return; }

	auto extract_trace = [&](int node) -> std::vector<double> {
		std::vector<double> trace(steps);
		int idx = netlist.node_to_idx[node];
		for (size_t k = 0; k < steps; k++) {
			trace[k] = wave[k][idx];
		}
		return trace;
	};
	
	std::vector<double> time = matplot::linspace(0.0, simconf.dt*static_cast<double>(steps - 1), steps);
	std::vector<double> data = extract_trace(1);
	auto graph_plotted = matplot::plot(time, data);
	matplot::hold(matplot::on);
	graph_plotted->display_name(std::format("V{}", 1));

	for (size_t i = 2; i < netlist.node_to_idx.size(); ++i) {
		data = extract_trace(i);
		graph_plotted = matplot::plot(time, data);
		graph_plotted->display_name(std::format("V{}", i));
	}

	matplot::xlabel("Time (s)");
	matplot::ylabel("Voltage (V)");
	matplot::legend({});

	matplot::hold(matplot::off);
	
	path_to_save.append("plot.jpg");
	matplot::save(path_to_save.c_str());
	if (simconf.plot) {
		matplot::show();
	}
}
