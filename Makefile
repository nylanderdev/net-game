all: netgame/launcher.jar netgame/netgame.exe

netgame/netgame.exe: netgame
	cargo build --release
	cp target/release/netgame.exe netgame/

netgame/launcher.jar: netgame
	mvnw package -f launcher/pom.xml
	cp launcher/target/launcher.jar netgame/

netgame:
	mkdir netgame