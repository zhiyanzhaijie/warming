UI_DIR := crates/ui

WARMING_LOG ?= warn,ui=info,api=info,app=info,infra=info,adapters=info

.PHONY: desktop web

desktop:
	cd $(UI_DIR) && APP_ENV=development RUST_LOG="$(WARMING_LOG)" dx serve -p desktop --desktop

web:
	cd $(UI_DIR) && APP_ENV=development RUST_LOG="$(WARMING_LOG)" dx serve -p web --web
