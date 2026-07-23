module spi_top_tb;
    logic clk;
    logic reset_n_mcu;
    logic reset_n_peri;

    logic start_send_mcu;
    logic start_recv_mcu;

    // --- tri-state bus ---
    wire  [7 : 0] dbus_mcu;   // shared wire — connects to spi_top inout port
    logic [7 : 0] tb_drive;   // TB's intended value
    logic       tb_oe;      // 1 = TB drives, 0 = release (let controller drive)

    assign dbus_mcu = (tb_oe) ? tb_drive : 8'bz;

    logic [7:0] tx_data_peri;
    logic [7:0] rx_data_peri;
    logic       rx_done_peri;
    logic       tx_done_peri;

    // clock
    initial clk = 0;
    always #10 clk = ~clk;

    spi_top dut (
        .clk            (clk),
        .reset_n_mcu    (reset_n_mcu),
        .reset_n_peri   (reset_n_peri),
        .start_send_mcu (start_send_mcu),
        .start_recv_mcu (start_recv_mcu),
        .dbus_mcu       (dbus_mcu),
        .tx_data_peri   (tx_data_peri),
        .rx_data_peri   (rx_data_peri),
        .rx_done_peri   (rx_done_peri),
        .tx_done_peri   (tx_done_peri)
    );

    initial begin
        $dumpfile("spi_top_tb.vcd");
        $dumpvars(0, spi_top_tb);

        reset_n_mcu    = 0;
        reset_n_peri   = 0;
        start_send_mcu = 0;
        start_recv_mcu = 0;
        tb_drive       = 8'd0;
        tb_oe          = 1'b1;  
        tx_data_peri   = 8'hA5;

        @(negedge clk);
        @(posedge clk);
        reset_n_mcu  = 1;
        reset_n_peri = 1;

        // TEST 1: controller sends 0xF4 to peripheral
        repeat(2) @(posedge clk);
        tb_drive = 8'hF4;
        tb_oe    = 1'b1;
        @(posedge clk);
        start_send_mcu = 1;
        @(negedge clk);
        start_send_mcu = 0;

        @(posedge rx_done_peri);
		if (rx_data_peri === 8'hF4)
		  $display("TEST 1 PASS: peripheral got 0x%02h = 0x%08b", rx_data_peri, rx_data_peri);
		else
		  $display("TEST 1 FAIL: peripheral got 0x%02h = 0x%08b (expected 0xF4 = 0x%08b)",
	  		rx_data_peri, rx_data_peri, 8'hF4);

        // TEST 2: controller reads from peripheral
        repeat(2) @(posedge clk);
        tb_oe          = 1'b0;   // release bus — controller will drive it
        start_recv_mcu = 1;
        @(posedge clk);
        start_recv_mcu = 0;

        // wait for controller to finish and put data on dbus
		// Wait for controller to assert bus_oe (it does this the cycle it sees finish_rx)
		@(posedge dut.mcu.bus_oe);
		@(posedge clk);  // let the assign propagate
		if (dbus_mcu === 8'hA5)
			$display("TEST 2 PASS: controller read 0x%02h", dbus_mcu);
		else
			$display("TEST 2 FAIL: got 0x%02h (expected 0xA5)", dbus_mcu);

        // reclaim bus
        tb_oe    = 1'b1;
        tb_drive = 8'd0;

        repeat(4) @(posedge clk);
        $finish;
    end
endmodule
