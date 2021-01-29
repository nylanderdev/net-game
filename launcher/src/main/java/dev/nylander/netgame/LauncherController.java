package dev.nylander.netgame;

import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.TextField;

import java.io.IOException;
import java.io.InputStream;
import java.util.Optional;

public class LauncherController {
    Optional<InputStream> netGameErr;

    @FXML
    TextField addressField;

    @FXML
    Button hostButton;

    @FXML
    Button joinButton;

    @FXML
    private void launchAsHost() throws IOException {
        Runtime runtime = Runtime.getRuntime();
        Process netGame = runtime.exec("netgame -host");
        //InputStream netGameErr = netGame.getErrorStream();
        //this.netGameErr = Optional.of(netGameErr);
    }

    @FXML
    private void launchAsGuest() throws IOException {
        // todo: escape address properly
        String address = addressField.getText().split("\\s")[0];
        Runtime runtime = Runtime.getRuntime();
        runtime.exec(String.format("netgame \"%s\"", address));
    }
}
