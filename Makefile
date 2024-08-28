CC=gcc
SRC=$(wildcard hdmi*.c)
OBJ=$(SRC:.c=.o)
CFLAGS=-g
LDFLAGS=-lusb-1.0 -lhidapi-hidraw
BINARY=kvm_switcher
BINARY2=hdmi_kvm_switch

all: $(BINARY) $(BINARY2)

%.o: %.c
	$(CC) -c -o $@ $< $(CFLAGS) $(LDFLAGS)

$(BINARY): $(OBJ)
	$(CC) -o $@ $^ $(CFLAGS) $(LDFLAGS)

$(BINARY2): $(OBJ)
	$(CC) -o $@ $^ $(CFLAGS) $(LDFLAGS)

release: CFLAGS=-Os -ffunction-sections -fdata-sections
release: LDFLAGS+=-Wl,--gc-sections
release: clean $(BINARY)
	strip $(BINARY)

.PHONY: clean
clean:
	rm -f $(OBJ) $(BINARY)
