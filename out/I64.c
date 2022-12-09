#include "I64.h"
I64* create(long value) { 
	I64* self = malloc(sizeof(I64));
	((self->inner) = value);
}
