TARGET_DIR := $(shell cargo metadata --format-version 1 --no-deps | jq -r '.target_directory')
COMP := $(TARGET_DIR)/release/qcc
COMPFLAGS = --print-ast --print-qasm
AS = $(COMP)

main: toss.ql math.ql std.ql
	$(COMP) $(COMPFLAGS) toss.ql

.DEFAULT_GOAL:=all
all: main

clean:
	rm -f *.s
