module uart_top#(
		parameter SYS_CLK = 50_000_000,
		parameter BAUD_RATE = 9600
	)(
    // system
    input  logic        clk,
    input  logic        reset_n,
 
    // frame config (static while a transfer is in progress)
    input  logic        parity,     // 0 = no parity bit, 1 = even-parity bit
    input  logic        stop,       // 0 = 1 stop bit,    1 = 2 stop bits
 
    // tx side
    input  logic        start,      // pulse for one baud_clk cycle to begin TX
    input  logic [7:0]  send_data,
    output logic        tx,
 
    // rx side
    input  logic        rx,
    output logic [7:0]  receive_data,
    output logic        finish,     // high for one baud_clk cycle when RX completes
    output logic        parity_error,
 
    // debug only – remove before synthesis
    output logic        o_baud_clk
);

	logic baud_clk;

	baud_gen #(
		.SYS_CLK 	(SYS_CLK),
		.BAUD_RATE	(BAUD_RATE)
		) baud_generator(
		.clk		(clk),
		.reset_n	(reset_n),
		.baud_clk	(baud_clk)
	);

	uart_tx u_tx(
		.reset_n			(reset_n),
		.clk				(baud_clk),
		.start				(start),
		.data_frame			(send_data),
		.parity				(parity),
		.stop				(stop),
		.tx					(tx)
	);

	uart_rx u_rx(
		.reset_n     	(reset_n),
		.clk         	(baud_clk),
		.parity      	(parity),
		.stop        	(stop),
		.rx          	(rx),
		.finish      	(finish),
		.parity_error	(parity_error),
		.data_frame  	(receive_data)
	);

	assign o_baud_clk = baud_clk;

endmodule
