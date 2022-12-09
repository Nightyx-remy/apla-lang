#include "I8.h"
I8* create(byte value) { 
	I8* self = malloc(sizeof(I8));
	((self->inner) = value);
}
