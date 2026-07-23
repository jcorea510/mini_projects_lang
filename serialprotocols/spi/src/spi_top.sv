module spi_top(
	input  logic clk,
	input  logic reset_n_mcu,
	input  logic reset_n_peri,
	
	// cpu interface
	input  logic start_send_mcu,
	input  logic start_recv_mcu,
	inout  wire [7 : 0] dbus_mcu, // shared data bus
 
    // spi data interface
    input  logic [7:0]  tx_data_peri,
    output logic [7:0]  rx_data_peri,
    output logic        rx_done_peri,
    output logic        tx_done_peri
);
 
logic poci;
logic pico;
logic cs_n;
logic sclk;
 
spi_controller mcu(
	.clk			(clk),
	.reset_n		(reset_n_mcu),
	.start_send		(start_send_mcu),
	.start_recv		(start_recv_mcu),
	.dbus			(dbus_mcu),
	.poci			(poci),
	.pico			(pico),
	.sclk 			(sclk),
	.cs_n			(cs_n)
);
 
spi_peripheral peripheral(
	.clk			(sclk),
    .reset_n		(reset_n_peri),
    .cs_n			(cs_n),     
    .pico			(pico),     
    .poci			(poci),     
    .tx_data		(tx_data_peri),
    .rx_data		(rx_data_peri),
    .rx_done		(rx_done_peri),
    .tx_done		(tx_done_peri)
);
 
endmodule
