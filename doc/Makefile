SHELL = /bin/sh
SOURCES = $(shell echo *.gv)
TARGETS = $(SOURCES:.gv=.png)

%.png: %.gv
	dot -T png -o $@ $^

all: $(TARGETS)

clean:
	rm -f $(TARGETS)
