#include <filesystem>
#include <netlist.hpp>
#include <components.hpp>
#include <nlohmann/json.hpp>
#include <fstream>
#include <print>
#include <string>
#include <algorithm>

using json = nlohmann::json;

static const std::unordered_map<std::string, CompType> typeMap = {
    {"resistor", CompType::Resistor},
    {"capacitor", CompType::Capacitor},
    {"inductor", CompType::Inductor},
    {"voltage_source", CompType::VoltageSource}
};

NetList readNetlist(std::filesystem::path &filename) {
	NetList netlist;
	netlist.max_nodes = 0;

	std::fstream file(filename);
	json data = json::parse(file);
	file.close();
	
	setVectorComponents(netlist, data);
	setNodeToIndex(netlist);

	return netlist;
}

void setVectorComponents(NetList &netlist, json &data) {
	auto components = data["components"];

	for (auto &kv: components.items()) {
		const auto name = kv.key();
		const auto obj = kv.value();

		std::string component_type_str = obj["type"];
		auto nodes = obj["nodes"].get<std::vector<int>>();
		double value = obj["value"];

		CompType component_type = CompType::Unknown;
		auto it = typeMap.find(component_type_str);
		if (it != typeMap.end()) {
			component_type = it->second;
		}

		switch (component_type) {
			case CompType::Resistor: {
				netlist.components.push_back({CompType::Resistor, {nodes[0], nodes[1]}, value, name});
				netlist.max_nodes = std::max(netlist.max_nodes, nodes[0]);
				netlist.max_nodes = std::max(netlist.max_nodes, nodes[1]);
				break;
			}
			case CompType::Capacitor: {
				netlist.components.push_back({CompType::Capacitor, {nodes[0], nodes[1]}, value, name});
				netlist.max_nodes = std::max(netlist.max_nodes, nodes[0]);
				netlist.max_nodes = std::max(netlist.max_nodes, nodes[1]);
				break;
			}
			case CompType::Inductor: {
				netlist.components.push_back({CompType::Inductor, {nodes[0], nodes[1]}, value, name});
				netlist.max_nodes = std::max(netlist.max_nodes, nodes[0]);
				netlist.max_nodes = std::max(netlist.max_nodes, nodes[1]);
				break;
			}
			case CompType::VoltageSource: {
				netlist.components.push_back({CompType::VoltageSource, {nodes[0], nodes[1]}, value, name});
				netlist.max_nodes = std::max(netlist.max_nodes, nodes[0]);
				netlist.max_nodes = std::max(netlist.max_nodes, nodes[1]);
				break;
			}
			case CompType::Unknown: {
				break;
			}
		}
	}
}

void setNodeToIndex(NetList &netlist) {
	netlist.node_to_idx = std::vector<int>(netlist.max_nodes + 1, -1);
	int idx = 0;
	for (int n = 1; n <= netlist.max_nodes; ++n) {
		netlist.node_to_idx[n] = idx++;
	}
	int volt_unknows = idx;
	int extra_unknows = 0;
	for (auto &comp: netlist.components) {
		if (comp.type == CompType::VoltageSource || comp.type == CompType::Inductor) {
			extra_unknows += 1;
		}
	}

	int total_unknows = volt_unknows + extra_unknows;
	netlist.volt_unknows = volt_unknows;
	netlist.extra_unknows = extra_unknows;
	netlist.total_unknows= total_unknows;

    std::println("Volt unknowns: {}", netlist.volt_unknows);
	std::println("Extras unknowns: {}", netlist.extra_unknows);
	std::println("Total unknowns: {}", netlist.total_unknows);
}

SimConf readSimConfig(std::filesystem::path &filename) {
	SimConf simconfig;

	std::fstream file(filename);
	json data = json::parse(file);
	file.close();

	auto simulation_confg = data["analysis"];
	for (auto &kv: simulation_confg.items()) {
		auto &obj = kv.value();

		simconfig.steps = obj["steps"];
		simconfig.dt = obj["dt"];
		simconfig.plot = obj["plot"];
	}
	
	return simconfig;
}
