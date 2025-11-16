#include <format>
#include <netlist.hpp>
#include <components.hpp>
#include <Eigen/Core>
#include <print>
#include <simulator.hpp>
#include <sstream>

MNA defineMNA(NetList netlist) {
    Eigen::MatrixXd G = Eigen::MatrixXd::Zero(netlist.total_unknows, netlist.total_unknows);
    Eigen::MatrixXd C = Eigen::MatrixXd::Zero(netlist.total_unknows, netlist.total_unknows);
    Eigen::VectorXd b = Eigen::VectorXd::Zero(netlist.total_unknows);
	MNA mna {G, C, b};

    // extraStartIndex is the index of the first extra variable in the unknown vector
    int extraStartIndex = netlist.volt_unknows;

    // stamp all components
    stamp_all(netlist, mna, extraStartIndex);
	return mna;
}

std::vector<Eigen::VectorXd> simulate_backward_euler(
		MNA &mna,
		NetList &netlist,
		SimConf &simconf)
{
	Eigen::VectorXd x0 = Eigen::VectorXd::Zero(netlist.total_unknows);
	int N = mna.G.rows();

	std::vector<Eigen::VectorXd> history;
	history.reserve(simconf.steps + 1);

	Eigen::MatrixXd M = mna.G + (mna.C / simconf.dt);
	Eigen::MatrixXd Minv;

	Eigen::VectorXd x = x0;
	history.push_back(x);

	for (int k = 0; k < simconf.steps; k++) {
		Eigen::VectorXd rhs = mna.b + (mna.C / simconf.dt) * x;
		Eigen::VectorXd x_next = M.householderQr().solve(rhs);
		x = x_next;
		history.push_back(x);
	}

	std::println("Time simulation completed.");
	std::println("Final state:\n");

	std::stringstream ss;
	ss << history.back().transpose();
	std::println("{}\n", ss.str());

	// Optionally print per-step node voltages:
	for(int i = 0; i < history.size(); i++){
		double t = i * simconf.dt;
		std::print("t={:.4f},", t);
		for (int n = 1; n <= netlist.volt_unknows; n++) {
			char separator = (netlist.volt_unknows == n) ? '\n' : ',';
			std::print(" V{}={:.4f}{}", n, history[i][netlist.node_to_idx[n]], separator);
		}
	}
	return history;
}
