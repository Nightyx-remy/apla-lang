#include "I32.h"
I32* create(int value) { 
	I32* self = malloc(sizeof(I32));
	((self->inner) = value);
}
