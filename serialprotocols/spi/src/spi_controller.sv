module spi_controller(
	input  logic clk,
	input  logic reset_n,
	
	// cpu interface
	input  logic start_send,
	input  logic start_recv,
	inout  wire [7 : 0] dbus, // shared data bus
 
	// spi lines to peripheral
	input  logic poci,
	output logic pico,
	output logic sclk,
	output logic cs_n
);
	// tri-state bus control
	logic bus_oe;
	logic [7 : 0] bus_drive;
 
	assign dbus = (bus_oe) ? bus_drive : 8'bz;
	
	// submodule signals
	logic [7 : 0] received_data;
	logic finish_rx;
 
	logic [7 : 0] tx_data;
	logic finish_tx;
	logic peri_cs;
	
 
	spi_rx rx_inst(
		.clk		(clk),
		.reset_n	(reset_n),
		.cs			(peri_cs),
		.din		(poci),
		.dout		(received_data),
		.finish		(finish_rx)
	);
	
	spi_tx tx_inst(
		.clk		(clk),
		.reset_n	(reset_n),
		.cs			(peri_cs),
		.din		(tx_data),
		.dout		(pico),
		.finish		(finish_tx)
	);
 
	typedef enum logic [1 : 0] {
		IDLE = 2'b11,
		READING = 2'b00,
		SENDING = 2'b01
	} state_t;
	state_t state;
 
	always_ff@(posedge clk or negedge reset_n) begin
		if (!reset_n) begin
			state <= IDLE;	
			tx_data <= 8'd0;
			peri_cs <= 1'b1;
			bus_oe <= 1'b0;
			bus_drive <= 8'd0;
			cs_n <= 1'b1;
		end else begin
			case (state)
				IDLE: begin
					bus_oe <= 1'b0;
					peri_cs <= 1'b1;
					cs_n <= 1'b1;
 
					if (start_send) begin
						tx_data <= dbus;
						state <= SENDING;
					end else if (start_recv) begin
						state <= READING;
					end
				end
 
				SENDING: begin
					peri_cs <= 1'b0;
					cs_n <= 1'b0;
					bus_oe <= 1'b0;
					if (finish_tx) begin
						peri_cs <= 1'b1;
						cs_n <= 1'b1;
						state <= IDLE;
					end
				end
 
				READING: begin
					peri_cs <= 1'b0;
					cs_n <= 1'b0;
					if (finish_rx) begin
						peri_cs <= 1'b1;
						cs_n <= 1'b1;
						bus_drive <= received_data;
						bus_oe <= 1'b1;
						state <= IDLE;
					end
				end
 
				default: begin
					state <= IDLE;
				end
			endcase
		end
	end
 
	assign sclk = clk;
endmodule
