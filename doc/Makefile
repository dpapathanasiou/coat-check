SHELL = /bin/sh
SOURCES = $(shell echo *.dot)
TARGETS = $(SOURCES:.dot=.png)

%.png: %.dot
	dot -T png -o $@ $^

all: $(TARGETS)

clean:
	rm -f $(TARGETS)
