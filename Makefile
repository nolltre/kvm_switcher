CC=gcc
SRC=$(wildcard *.c)
OBJ=$(SRC:.c=.o)
CFLAGS=-g
LDFLAGS=-lusb-1.0
BINARY=kvm_switcher

all: $(BINARY)

%.o: %.c
	$(CC) -c -o $@ $< $(CFLAGS) $(LDFLAGS)

$(BINARY): $(OBJ)
	$(CC) -o $@ $^ $(CFLAGS) $(LDFLAGS)

release: CFLAGS=-Os -ffunction-sections -fdata-sections
release: LDFLAGS+=-Wl,--gc-sections
release: clean $(BINARY)
	strip $(BINARY)

.PHONY: clean
clean:
	rm -f $(OBJ) $(BINARY)
