module spi_tx (
    input  logic        clk,
    input  logic        reset_n,
    input  logic        cs,
    input  logic [7:0]  din,
    output logic        dout,
    output logic        finish
);
    logic [2:0] bit_counter;
    logic [7:0] shreg;
 
    always_ff @(posedge clk or negedge reset_n) begin
        if (!reset_n) begin
            bit_counter <= 3'd0;
            shreg       <= 8'd0;
            dout        <= 1'b0;
            finish      <= 1'b0;
        end else if (cs) begin
            shreg       <= din;      // preload every idle cycle
            bit_counter <= 3'd0;
            dout        <= 1'b0;
            finish      <= 1'b0;
        end else begin
            // Drive current bit and always advance the counter.
            dout   <= shreg[bit_counter];
            finish <= 1'b0;
            if (bit_counter == 3'd7) begin
                bit_counter <= 3'd0;
                finish      <= 1'b1;
            end else begin
                bit_counter <= bit_counter + 3'd1;
            end
        end
    end
endmodule
