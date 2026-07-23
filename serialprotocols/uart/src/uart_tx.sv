// Moore FSM that serialises one UART frame onto tx.
//
// Frame structure (LSB first):
//   [start=0] [d0..d7] [parity?] [stop x1 or x2]
//
// DESIGN NOTE:
//   start is sampled on baud_clk (16x oversampled clock), not sys_clk.
//   The caller must hold start high for at least one full baud_clk period
//   (~104 µs at 9600 baud with 50 MHz sys_clk) to guarantee it is seen.

module uart_tx(
	input  logic 		 reset_n,
	input  logic 		 clk, 			// baud_clk (16x baud rate)
	input  logic 		 start, 		// start data frame transmition
	input  logic [7 : 0] data_frame,	// fixed data frame of 8 bits
	input  logic		 parity, 		// 0 = no parity, 1 = even parity
	input  logic		 stop, 			// 0 = 1 stop bit, 1 = 2 stop bits
	output logic		 tx
);

	typedef enum logic [2 : 0] {
		IDLE = 3'b000,
		START_BIT = 3'b001,
		DATA_FRAME = 3'b010,
		PARITY = 3'b011,
		STOP = 3'b100
	} state_t;

	state_t current_state, next_state;
	logic [3 : 0] frame_counter; 		// count bits 0...7 during DATA_FRAME
	logic stop_bit_sent; 				// flag: have we already spent one cycle in stop?

    // Block 1: Sequential (states register and counters)
	always_ff @(posedge clk or negedge reset_n) begin
        if (!reset_n) begin
            current_state <= IDLE;
			frame_counter <= 0;
			stop_bit_sent <= 0;
        end else begin
            current_state <= next_state;

			case (current_state) 
				DATA_FRAME: begin
					if (frame_counter == 4'd7) begin
						frame_counter <= 4'd0;
					end else begin
						frame_counter <= frame_counter + 4'd1;
					end
				end
				STOP: begin
					stop_bit_sent <= stop_bit_sent + 1'b1;
				end
				default: begin
					frame_counter <= 4'd0;
					stop_bit_sent <= 1'b0;
				end
			endcase
        end
    end
	
	// Block 2: Combinational (next state logic)
	always_comb begin
        next_state = current_state; 
		case (current_state)
			IDLE: begin
				if (start) begin
					next_state = START_BIT;
				end
			end
			START_BIT: begin
				next_state = DATA_FRAME;
			end
			
			DATA_FRAME: begin
				if (frame_counter == 4'd7) begin
					if (!parity) begin
						next_state = STOP;
					end else begin
						next_state = PARITY;
					end
				end
			end

			PARITY: begin
				next_state = STOP;
			end

			STOP: begin
				if (!stop) begin
					next_state = IDLE;
				end else begin
					if (stop_bit_sent) begin
						next_state = IDLE;
					end
				end
			end

			default: begin
				next_state = IDLE;
			end
		endcase
	end
	
	// Block 3: Output logic
	always_comb begin
		case(current_state)
			START_BIT:
				tx = 1'b0;
			DATA_FRAME:
				tx = data_frame[frame_counter];
			PARITY:
				tx = ^data_frame; // even parity
			STOP:
				tx = 1'b1; // IDLE and STOP are HIGH
		endcase
	end
endmodule
