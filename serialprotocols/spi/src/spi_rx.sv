module spi_rx (
    input  logic        clk,
    input  logic        reset_n,
    input  logic        cs,
    input  logic        din,
    output logic [7:0]  dout,
    output logic        finish
);
    logic [2:0] bit_counter;
    logic [7:0] frame;
    logic       first;
 
    always_ff @(negedge clk or negedge reset_n) begin
        if (!reset_n) begin
            bit_counter <= 3'd0;
            frame       <= 8'd0;
            finish      <= 1'b0;
            first       <= 1'b1;
        end else if (cs) begin
            bit_counter <= 3'd0;
            finish      <= 1'b0;
            first       <= 1'b1;
        end else if (first) begin
            // Skip the first negedge: TX fires on posedge so bit 0 is not yet
            // on the wire. Just clear the flag; no frame write, no counter move.
            first <= 1'b0;
        end else begin
            // Sample din into the current bit slot and advance the counter.
            frame[bit_counter] <= din;
            finish <= 1'b0;
            if (bit_counter == 3'd7) begin
                bit_counter <= 3'd0;
                finish      <= 1'b1;
            end else begin
                bit_counter <= bit_counter + 3'd1;
            end
        end
    end
 
    assign dout = frame;
endmodule
