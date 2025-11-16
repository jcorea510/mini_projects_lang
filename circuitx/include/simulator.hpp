#pragma once

#include <Eigen/Core>
#include <Eigen/Dense>
#include <vector>
#include <netlist.hpp>

struct MNA{
	Eigen::MatrixXd G;
	Eigen::MatrixXd C;
	Eigen::VectorXd b;
};

MNA defineMNA(NetList netlist);

// performs AC trasient response using backward euler method
std::vector<Eigen::VectorXd> simulate_backward_euler(
		MNA &mna,
		NetList &netlist,
		SimConf &simconfig
);
