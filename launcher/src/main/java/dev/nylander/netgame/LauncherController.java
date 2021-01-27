package dev.nylander.netgame;

import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.TextField;

import java.io.IOException;

public class LauncherController {
    @FXML
    TextField addressField;

    @FXML
    Button hostButton;

    @FXML
    Button joinButton;

    @FXML
    private void launchAsHost() throws IOException {
        Runtime runtime = Runtime.getRuntime();
        runtime.exec("netgame -host");
    }

    @FXML
    private void launchAsGuest() throws IOException {
        // todo: escape address properly
        String address = addressField.getText().split("\\s")[0];
        Runtime runtime = Runtime.getRuntime();
        runtime.exec(String.format("netgame \"%s\"", address));
    }
}
