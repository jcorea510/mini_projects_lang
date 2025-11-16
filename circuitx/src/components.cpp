#include <netlist.hpp>
#include <simulator.hpp>
#include <components.hpp>
#include <cstdio>
#include <print>
#include <sstream>

static double inv(double v) { 
	return 1.0/v;
}

void stamp_component(
	const Component& component,
	MNA &mna,
	NetList &netlis,
	int &nextExtraIndex)
{
	auto index_of = [&](int node)->int {
		return netlis.node_to_idx[node];
	};

	switch (component.type) {
		case CompType::Resistor: {
			int n1 = index_of(component.nodes[0]);
			int n2 = index_of(component.nodes[1]);

			double g = inv(component.value);

			if (n1 >= 0) { mna.G(n1, n1) += g;}
			if (n2 >= 0) { mna.G(n2, n2) += g;}
			if (n1 >= 0 && n2 >= 0) { mna.G(n1, n2) -= g; mna.G(n2, n1) -= g;}
			break;
		}
		case CompType::Capacitor: {
			int n1 = index_of(component.nodes[0]);
			int n2 = index_of(component.nodes[1]);

			double cap = component.value;

			if (n1 >= 0) { mna.C(n1, n1) += cap; }
			if (n2 >= 0) { mna.C(n2, n2) += cap; }
			if (n1 >= 0 && n2 >= 0) { mna.C(n1, n2) -= cap; mna.C(n2, n1) -= cap; }
			break;
		}
		case CompType::Inductor: {
			int extra = nextExtraIndex++;
			int npos = index_of(component.nodes[0]);
			int nneg = index_of(component.nodes[1]);

			if (npos >= 0) { mna.G(npos, extra) += 1.0; mna.G(extra, npos) += 1.0; }
			if (nneg >= 0) { mna.G(nneg, extra) -= 1.0; mna.G(extra, nneg) -= 1.0; }

			mna.C(extra, extra) += component.value;
			break;
		}
		case CompType::VoltageSource: {
			int extra = nextExtraIndex++;
			int npos = index_of(component.nodes[0]);
			int nneg = index_of(component.nodes[1]);

			if (npos >= 0) { mna.G(npos, extra) += 1.0; mna.G(extra, npos) += 1.0; }
			if (nneg >= 0) { mna.G(nneg, extra) -= 1.0; mna.G(extra, nneg) -= 1.0; }
			mna.b(extra) += component.value;
			break;
		}
		default:
			std::println(stderr, "stamp_component: unknown type");
			// std::cerr<< "stamp_component: unknown type";
	}
}

void stamp_all(
		NetList &netlist,
		MNA &mna,
		int extraStartIndex)
{
	int nextExtra = extraStartIndex;
	for (const auto &component: netlist.components) {
		stamp_component(component, mna, netlist, nextExtra);
	}
	
	std::stringstream ss;
	ss << mna.G;
    std::println("G matrix:\n{}\n", ss.str());
	ss.clear();

	ss << mna.C;
    std::println("C matrix:\n{}\n", ss.str());
	ss.clear();

	ss << mna.b.transpose();
    std::println("b vector:\n{}\n", ss.str());
	ss.clear();
}

