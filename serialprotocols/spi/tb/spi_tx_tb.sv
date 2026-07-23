`timescale 1ns / 1ps

module spi_tx_tb;
	logic clk;
	logic reset_n;
	logic cs;
	logic [7 : 0] din;
	logic dout;
	logic finish;

	initial begin
		clk = 0;
		forever begin
			#10 clk = ~clk;
		end
	end
	
	// PICO module
	spi_tx dut(
		.clk			(clk),
		.reset_n		(reset_n),
		.cs				(cs),
		.din			(din),
		.dout			(dout),
		.finish			(finish)
	);

	initial begin
		$monitor("At time %t din: %h, tx: %h", $time, din, dout);
		$dumpfile("spi_tx_tb.vcd");
		$dumpvars(0, spi_tx_tb);

		reset_n = 0;
		cs = 1;
		din = 8'd0;
		repeat(4) @(negedge clk);
		reset_n = 1;
		cs = 0;

		din = 8'hA5;
		@(posedge finish);

		din = 8'hF4;
		@(posedge finish);
		repeat(2) @(posedge clk);

		$finish;
	end
endmodule
