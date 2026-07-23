`timescale 1ns / 1ps
module uart_rx_tb;

	// -------------------------------------------------------
	// DUT signals
	// -------------------------------------------------------
	logic        reset_n;
	logic        clk;
	logic        parity;
	logic        stop;
	logic        rx;
	logic        finish;
	logic        parity_error;
	logic [7:0]  data_frame;

	// -------------------------------------------------------
	// Clock: 50 MHz → 20 ns period
	// -------------------------------------------------------
	initial clk = 0;
	always #10 clk = ~clk;

	// -------------------------------------------------------
	// DUT instantiation
	// -------------------------------------------------------
	uart_rx dut(
		.reset_n     (reset_n),
		.clk         (clk),
		.parity      (parity),
		.stop        (stop),
		.rx          (rx),
		.finish      (finish),
		.parity_error(parity_error),
		.data_frame  (data_frame)
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
		@(negedge clk); rx = 0;
		// 8 data bits, LSB first
		for (i = 0; i < 8; i++) begin
			@(negedge clk); rx = data_byte[i];
		end
		// Optional parity (even parity)
		if (use_parity) begin
			par = ^data_byte;           // even parity: XOR of all bits
			@(negedge clk); rx = par;
		end
		// Stop bit(s)
		@(negedge clk); rx = 1;
		if (two_stop) begin
			@(negedge clk); rx = 1;
		end
	endtask

	// -------------------------------------------------------
	// Stimulus
	// -------------------------------------------------------
	initial begin
		$dumpfile("uart_rx.vcd");
		$dumpvars(0, uart_rx_tb);

		// Initialise
		reset_n = 0;
		rx      = 1;   // idle (high)
		parity  = 0;
		stop    = 0;
		@(negedge clk);
		@(negedge clk);
		reset_n = 1;
		@(negedge clk);

		// ---------------------------------------------------
		// Test 1: 8'hA5, no parity, 1 stop bit
		// Expected data_frame = 8'hA5 = 1010_0101
		// ---------------------------------------------------
		parity = 0; stop = 0;
		send_uart(8'hA5, 0, 0);

		// Wait for finish flag
		@(posedge finish);
		@(negedge clk);
		$display("[TEST 1] data_frame=0x%02h  finish=%b  parity_error=%b  (expect A5, 1, 0)",
		          data_frame, finish, parity_error);

		// Gap between frames
		repeat(3) @(negedge clk);

		// ---------------------------------------------------
		// Test 2: 8'hA5, even parity, 2 stop bits
		// ---------------------------------------------------
		parity = 1; stop = 1;
		send_uart(8'hA5, 1, 1);

		@(posedge finish);
		@(negedge clk);
		$display("[TEST 2] data_frame=0x%02h  finish=%b  parity_error=%b  (expect A5, 1, 0)",
		          data_frame, finish, parity_error);

		repeat(3) @(negedge clk);

		// ---------------------------------------------------
		// Test 3: 8'h00
		// ---------------------------------------------------
		parity = 0; stop = 0;
		send_uart(8'h00, 0, 0);

		@(posedge finish);
		@(negedge clk);
		$display("[TEST 3] data_frame=0x%02h  finish=%b  parity_error=%b  (expect 00, 1, 0)",
		          data_frame, finish, parity_error);

		repeat(3) @(negedge clk);

		// ---------------------------------------------------
		// Test 4: 8'hFF
		// ---------------------------------------------------
		parity = 0; stop = 0;
		send_uart(8'hFF, 0, 0);

		@(posedge finish);
		@(negedge clk);
		$display("[TEST 4] data_frame=0x%02h  finish=%b  parity_error=%b  (expect FF, 1, 0)",
		          data_frame, finish, parity_error);

		repeat(3) @(negedge clk);
		$finish;
	end

endmodule
