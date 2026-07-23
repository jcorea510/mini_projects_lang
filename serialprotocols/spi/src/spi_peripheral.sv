module spi_peripheral (
    input  logic        clk,
    input  logic        reset_n,
 
    // SPI lines from controller
    input  logic        cs_n,     
    input  logic        pico,     
    output logic        poci,     
 
    // Local data interface
    input  logic [7:0]  tx_data,  // byte to send back when queried
    output logic [7:0]  rx_data,  // byte received from controller
    output logic        rx_done,  // pulses 1 for one cycle when rx_data is valid
    output logic        tx_done
);
 
    spi_rx rx_inst (
        .clk     (clk),
        .reset_n (reset_n),
        .cs      (cs_n),
        .din     (pico),
        .dout    (rx_data),
        .finish  (rx_done)
    );
 
    spi_tx tx_inst (
        .clk     (clk),
        .reset_n (reset_n),
        .cs      (cs_n),
        .din     (tx_data),
        .dout    (poci),
        .finish  (tx_done)
    );
 
endmodule
