.PHONY: all clean

CC:= g++

CFLAGS = -std=c++17 -Wno-deprecated-enum-enum-conversion -O2
# CFLAGS += `pkg-config --cflags opencv4` 
# LDFLAGS = `pkg-config --libs opencv4`


SOURCES := $(wildcard *.cpp)
OBJECTS := $(patsubst %.cpp,%.o,$(SOURCES))
DEPENDS := $(patsubst %.cpp,%.d,$(SOURCES))

all: main

clean:
	$(RM) $(OBJECTS) $(DEPENDS) main

-include $(DEPENDS)

# Make object files
%.o: %.cpp Makefile
	$(CC) $(WARNING) $(CFLAGS) -MMD -MP -c $< -o $@

# Linking the executable from the object files
main: $(OBJECTS)
	$(CC) $^ $(LDFLAGS) -o $@
	./main
