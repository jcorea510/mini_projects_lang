#pragma once

#include <Eigen/Core>
#include <array>
#include <string>
#include <Eigen/Dense>
#include <vector>
#include <netlist.hpp>
#include <simulator.hpp>

// strong typed and scoped enum
// contrary to classic enum
enum class CompType {
	Resistor,
	Capacitor,
	VoltageSource,
	Inductor,
	Unknown
};

struct Component {
	CompType type;
	// two endpoint by default: (positive node and negative node)
	// use 0 for ground always
	std::array<int, 2> nodes;
	// generic value of the component: simple by the moment, 
	// new types may requiere complex values.
	double value;
	// for debuggin
	std::string name;
};

// stamping interface: stamp a component into matrices
// G: conductance matrix,
// C: dynamic matrix (for capacitors/inductors),
// b: RHS vector (sources),
// index_map: map of node -> unknown index
// extraVarIndexStart: first index assigned to extra current unknowns;
// stamp should use and/or increment nextExtra (passed by reference) when it consumes a new "extra" unknown
void stamp_component(
	const Component& component,
	MNA &mna,
	NetList &netlis,
	int &nextExtraIndex
);

void stamp_all(
	NetList &netlist,
	MNA &mna,
	int extraStartIndex
);
