module spi_rx_tb;
	logic clk;
	logic reset_n;
	logic cs;
	logic din;
	logic [7 : 0] dout;
	logic finish;

	initial begin
		clk = 0;
		forever begin
			#10 clk = ~clk;
		end
	end
	
	// PICO module
	spi_rx dut(
		.clk			(clk),
		.reset_n		(reset_n),
		.cs				(cs),
		.din			(din),
		.dout			(dout),
		.finish			(finish)
	);

	task send_spi_data(
		input  logic [7 : 0] dsent
	);
		integer i;
		for (i = 0; i < 8; i = i + 1) begin
			din = dsent[i];
			@(negedge clk);
		end
	endtask

	initial begin
		$monitor("At time %t din: %h, rx: %h", $time, din, dout);
		$dumpfile("spi_rx_tb.vcd");
		$dumpvars(0, spi_rx_tb);

		reset_n = 0;
		cs = 1;
		din = 0;
		repeat(4) @(negedge clk);
		reset_n = 1;
		cs = 0;

		send_spi_data(8'hA5);
		repeat(2) @(posedge clk);

		$finish;
	end
endmodule
