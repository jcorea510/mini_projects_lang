#pragma once

#include <filesystem>
#include <netlist.hpp>
#include <vector>
#include <Eigen/Dense>

void time_ploter(
		std::vector<Eigen::VectorXd> &wave,
		NetList &netlist,
		SimConf &simconf,
		std::filesystem::path &path_to_save);
