#pragma once

#include <filesystem>
#include <nlohmann/json.hpp>
#include <vector>

struct Component;

using json = nlohmann::json;

struct NetList {
	std::vector<Component> components;
	std::vector<int> node_to_idx;
	int max_nodes;
	int volt_unknows;
	int extra_unknows;
	int total_unknows;
};

struct SimConf {
	double dt;
	int steps;
	bool plot;
};

NetList readNetlist(std::filesystem::path &filename);
void setVectorComponents(NetList &netlist, json &data);
void setNodeToIndex(NetList &netlist);
SimConf readSimConfig(std::filesystem::path &filename);
