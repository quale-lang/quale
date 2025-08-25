COMP = ~/workspace/quale-lang/quale/target/debug/qcc
COMPFLAGS = --print-ast --print-qasm
AS = $(COMP)

main: toss.ql math.ql std.ql
	$(COMP) $(COMPFLAGS) toss.ql

all: main

clean:
	rm -f *.s
