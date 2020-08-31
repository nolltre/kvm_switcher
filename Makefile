CC=gcc
SRC=$(wildcard *.c)
OBJ=$(SRC:.c=.o)
CFLAGS=-g
LDFLAGS=-lusb-1.0

all: kvm_switcher

%.o: %.c
	$(CC) -c -o $@ $< $(CFLAGS) $(LDFLAGS)

kvm_switcher: $(OBJ)
	$(CC) -o $@ $^ $(CFLAGS) $(LDFLAGS)

.PHONY: clean
clean:
	rm -f $(OBJ) kvm_switcher
