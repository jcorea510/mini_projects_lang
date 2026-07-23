`timescale 1ns / 1ps

module baud_gen_tb;
	logic clk;
	logic reset_n;
	logic baud_clk;


	// 50 MHz clock generator: toggles every 10 ns for a 20 ns period
	// Default system clock is 50 MHz an its the same as clk in dut
	initial begin
		clk = 0;
		forever begin
			#10 clk = ~clk;
		end
	end

	initial begin
		#4 reset_n = 1;
		#6 reset_n = 0;
		#9 reset_n = 1;
		#12 reset_n = 0;
		#19 reset_n = 1;
		#100000;
		$stop;
	end

	baud_gen dut(
		.clk		(clk),
		.reset_n	(reset_n),
		.baud_clk	(baud_clk)
		);
	

	initial begin
		$monitor("At time %t, value=%h (%0d)", $time, baud_clk, baud_clk);	
		$dumpfile("baud_gen.vcd");
		$dumpvars(0, baud_gen_tb);

		#100000;
		$finish;
	end

endmodule
