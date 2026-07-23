`timescale 1ns / 1ps

module uart_tx_tb;
	logic 			reset_n;
	logic 			clk;		// baud rate
	logic  			start; 		// start data frame transmition
	logic  [7 : 0] 	data_frame; // fixed data frame of 8 bits
	logic 			parity; 	// 0 -> 0 bit of parity, 1 -> 1 bit of parity
	logic 			stop; 		// 0 -> 1 bit to stop, 1 -> 2 bits to stop
	logic 			tx;
	
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
		#1000;
		$stop;
	end
	
	uart_tx dut(
		.reset_n			(reset_n),
		.clk				(clk),
		.start				(start),
		.data_frame			(data_frame),
		.parity				(parity),
		.stop				(stop),
		.tx					(tx)
	);

	initial begin
		data_frame = 8'b10110010;
		parity = 0;
		stop = 1;
		#25 start = 1;
		#20 start = 1;

		$monitor("At time %t, value=%h (%0d)", $time, tx, tx);	
		$dumpfile("uart_tx.vcd");
		$dumpvars(0, uart_tx_tb);

		#1000;
		$finish;
	end
endmodule
