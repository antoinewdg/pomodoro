install:
	cargo build --release
	mkdir -p "${SYSTEMD_DIR}"
	ln -sf ${ROOT_DIR}/systemd/pomodoro.service "${SYSTEMD_DIR}/pomodoro.service"
	ln -sf ${ROOT_DIR}/target/release/pomodoro "${HOME}/.local/bin/pomodoro"
	systemctl --user daemon-reload
	systemctl --user enable pomodoro.service
	systemctl --user restart pomodoro.service

.PHONY: install

ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
SYSTEMD_DIR:=${HOME}/.config/systemd/user
