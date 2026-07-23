// uart_rx
// Moore FSM that deserialises one UART frame from rx.

module uart_rx(
	input  logic 		 reset_n,
	input  logic 		 clk, 			// baud_clk (16x baud_rate)
	input  logic		 parity, 		// 0 -> 0 bit of parity, 1 -> 1 bit of parity
	input  logic		 stop, 			// 0 -> 1 bit to stop, 1 -> 2 bits to stop

	input  logic		 rx,			// uart message

	output logic		 finish,		// high for one baud_clk cycle
	output logic		 parity_error,	// tell MCU if there ir parity (error)
	output logic [7 : 0] data_frame		// fixed data frame of 8 bits
);

	typedef enum logic [2 : 0] {
		IDLE = 3'b000,
		START    = 3'b001,   // wait one clock to skip the start bit
		DATA_FRAME = 3'b010,
		PARITY = 3'b011,
		STOP = 3'b100,
		STOP_BIT = 3'b101
	} state_t;

	state_t current_state, next_state;
	logic [3 : 0] frame_counter; 		// helper variable to count bits sended in data frame
	logic 		  stop_bit_seen; 		// helper variable to count bits in stop
	logic [7 : 0] received_bits; 		// helper variable that store bits as they arrive
	logic 		  parity_check;			// helper variable that store parity check

    // Block 1: Sequential (states register and counters)
	always_ff @(posedge clk or negedge reset_n) begin
        if (!reset_n) begin
            current_state <= IDLE;
            frame_counter <= 4'd0;
            stop_bit_seen <= 1'b0;
            received_bits <= 8'd0;
            parity_check  <= 1'b0;
        end else begin
            current_state <= next_state;

			case (current_state) 
                IDLE: begin
                    received_bits <= 8'd0;
                    parity_check  <= 1'b0;
                    frame_counter <= 4'd0;
                    stop_bit_seen <= 1'b0;
                end
				DATA_FRAME: begin
					received_bits[frame_counter] <= rx;
					if (frame_counter == 4'd7) begin
						frame_counter <= 4'd0;
					end
					else begin
						frame_counter <= frame_counter + 4'd1;
					end
				end
				PARITY: begin
					parity_check <= rx;
				end
				STOP: begin
                    stop_bit_seen <= 1'b1;
				end
				default: begin
                    frame_counter <= 4'd0;
                    stop_bit_seen <= 1'b0;
				end
			endcase
        end
    end
	
	// Block 2: Combinational (next state logic)
	always_comb begin
        next_state = current_state; 
		case (current_state)
			IDLE: begin
				if (!rx) begin
					next_state = START;
				end
			end
			START: begin
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
					next_state = STOP_BIT;
				end else begin
					if (stop_bit_seen) begin
						next_state = STOP_BIT;
					end
				end
			end
			STOP_BIT: begin
				next_state = IDLE;
			end
			default: begin
				next_state = IDLE;
			end
		endcase
	end

	// Block 3: Output logic
	always_comb begin
		finish = 1'b0;
		parity_error = 1'b0;
		data_frame = received_bits;

        if (current_state == STOP_BIT) begin
            finish = 1'b1;
            parity_error = parity ? (^received_bits != parity_check) : 1'b0;
        end
	end
endmodule
