`timescale 1ns / 1ps

module uart_top_tb;
	logic clk;
	logic reset_n;

	logic parity;
	logic stop;
	
	logic start;
	logic [7 : 0] send_data; // eg from pc to stm32
	logic tx;

	logic rx;
	logic [7 : 0] receive_data; // eg from stm32 to pc
	logic finish;
	logic parity_error;

	logic baud_clk;
		
	// clock: system clock 50 MHz -> 20 ns period
	initial clk = 0;
	always #10 clk = ~clk;

	uart_top #(
		.SYS_CLK	(50_000_000),
		.BAUD_RATE	(781250) // this will result in divide 4 (50 MHz/(781_250 Hz x 16) = 4)
		) dut(
		.clk			(clk),
		.reset_n		(reset_n),
		.parity			(parity),
		.stop			(stop),
		.start			(start),
		.finish			(finish),
		.parity_error	(parity_error),
		.send_data		(send_data),
		.tx				(tx),
		.rx				(rx),
		.receive_data	(receive_data),
		.o_baud_clk		(baud_clk)
	);

	// -------------------------------------------------------
	// Task: send one UART frame on rx
	//   data_byte : 8-bit payload
	//   use_parity: 1 → append even-parity bit
	//   two_stop  : 1 → send 2 stop bits
	// -------------------------------------------------------
	task send_uart(
		input logic [7:0] data_byte,
		input logic       use_parity,
		input logic       two_stop
	);
		integer i;
		logic par;
		// Start bit (low for one clock period)
		@(negedge baud_clk); rx = 0;
		@(negedge baud_clk); rx = 0;
		// 8 data bits, LSB first
		for (i = 0; i < 8; i++) begin
			@(negedge baud_clk); rx = data_byte[i];
		end
		// Optional parity (even parity)
		if (use_parity) begin
			par = ^data_byte;           // even parity: XOR of all bits
			@(negedge baud_clk); rx = par;
		end
		// Stop bit(s)
		@(negedge baud_clk); rx = 1;
		if (two_stop) begin
			@(negedge baud_clk); rx = 1;
		end
	endtask

	// simulation stimulus
	initial begin
		$dumpfile("uart_top.vcd");
		$dumpvars(0, uart_top_tb);

		// Default values
		reset_n   = 0;
		rx        = 1;   // idle high
		parity    = 0;
		stop      = 0;
		start     = 0;
		send_data = 8'h00;

		// Reset using system clk — baud_clk does not exist yet
		repeat(4) @(negedge clk);
		reset_n = 1;
 
		// Now wait for baud_clk to start toggling before using it
		@(posedge baud_clk);
		@(negedge baud_clk);

		// Test 1 – RX path: 8'hA5, no parity, 1 stop bit
		parity = 0; stop = 0;
		send_uart(8'hA5, 0, 0);
		@(posedge finish);
		@(negedge baud_clk);
		$display("[TEST 1] receive_data=0x%02h  finish=%b  parity_error=%b  (expect A5, 1, 0)",
		          receive_data, finish, parity_error);
		repeat(3) @(negedge baud_clk);

		// Test 2 – RX path: 8'h4F, no parity, 1 stop bit
		reset_n = 1;
		@(posedge baud_clk);
		@(negedge baud_clk);

		parity = 0; stop = 1;
		send_uart(8'h4F, 0, 0);
		@(posedge finish);
		@(negedge baud_clk);
		$display("[TEST 2] receive_data=0x%02h  finish=%b  parity_error=%b  (expect 4F, 1, 0)",
		          receive_data, finish, parity_error);
		repeat(3) @(negedge baud_clk);

		// Test 3 – TX path: assert start, wait for tx to go
		//           low (start bit), then idle again
		parity    = 0;
		stop      = 0;
		send_data = 8'hB2;
		@(negedge baud_clk);
		start = 1;
		@(negedge baud_clk);
		start = 0;
		// Wait until tx goes low (start bit) then high (stop bit)
		@(negedge tx);        // start bit
		@(posedge tx);        // stop/idle
		repeat(16) @(negedge baud_clk);

		$display("[TEST 3] TX frame for 0xB2 completed, tx=%b (expected 1)", tx);

		$finish;
	end
endmodule
