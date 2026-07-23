// Divides sys_clk by (SYS_CLK / (BAUD_RATE * 16)) to produce a baud_clk
// that toggles at 16x the target baud rate (oversampling for RX).

module baud_gen #(
	parameter SYS_CLK = 50_000_000,
	parameter BAUD_RATE = 9600
) (
	input logic  clk,
	input logic  reset_n,
	output logic baud_clk
);

	// divide the system clock by a baud rate and a over sampling amount (fixed 16)	
	localparam DIVISOR = SYS_CLK / (BAUD_RATE * 16);

	reg [$clog2(DIVISOR) - 1: 0] counter;

	always_ff@(posedge clk or negedge reset_n) begin
		if (!reset_n) begin
			counter <= 0;
			baud_clk <= 1'b0;
		end else begin
			if (counter == DIVISOR - 1) begin
				counter <= 0;
				baud_clk <= ~baud_clk;
			end else begin
				counter <= counter + 1;
			end
		end
	end

endmodule
