#include "I16.h"
I16* create(short value) { 
	I16* self = malloc(sizeof(I16));
	((self->inner) = value);
}
