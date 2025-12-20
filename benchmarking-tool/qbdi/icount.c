#include "QBDIPreload.h"

#include <dlfcn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <inttypes.h>

QBDIPRELOAD_INIT;

int qbdipreload_on_start(void *main) { 
	return QBDIPRELOAD_NOT_HANDLED; 
}

int qbdipreload_on_premain(void *gprCtx, void *fpuCtx) {
	return QBDIPRELOAD_NOT_HANDLED;
}

int qbdipreload_on_main(int argc, char **argv) {
	if (getenv("QBDI_DEBUG") != NULL) {
		qbdi_setLogPriority(QBDI_DEBUG);
	} else {
		qbdi_setLogPriority(QBDI_WARNING);
	}
	return QBDIPRELOAD_NOT_HANDLED;
}

typedef struct Entry {
	char *symbol;
	char *kind;
	uint64_t count;
	struct Entry *next;
} Entry;

#define HASH_SIZE 4096

static Entry *table[HASH_SIZE];

/// Fowler–Noll–Vo hash function I found somewhere
static uint64_t hash_pair(const char *a, const char *b) {
	uint64_t h = 1469598103934665603ULL;
	for (; *a; a++) h = (h ^ *a) * 1099511628211ULL;
	for (; *b; b++) h = (h ^ *b) * 1099511628211ULL;
	return h % HASH_SIZE;
}

/// increment value in the table.
static void increment(const char *sym, const char *kind) {
	uint64_t h = hash_pair(sym, kind);
	Entry *e = table[h];
	while (e) {
		// if existing entry then increment that value
		if (strcmp(e->symbol, sym) == 0 && strcmp(e->kind, kind) == 0) {
			e->count++;
			return;
		}
		e = e->next;
	}

	e = calloc(1, sizeof(Entry));
	e->symbol = strdup(sym);
	e->kind   = strdup(kind);
	e->count  = 1;
	e->next   = table[h];
	table[h]  = e;
}

static const char *classify(const InstAnalysis *ia) {
	// return ia->mnemonic;
	if (ia->isBranch) return "branch";
	if (ia->isCall)   return "call";
	if (ia->isReturn) return "return";
	if (ia->isCompare) return "compare";
	if (ia->mayLoad || ia->mayStore) return "memory";
	return ia->mnemonic;
	// return "other";
}

static VMAction onInstruction(VMInstanceRef vm, GPRState *gprState, FPRState *fprState, void *data) {
	const int flag = QBDI_ANALYSIS_INSTRUCTION | QBDI_ANALYSIS_OPERANDS | QBDI_ANALYSIS_SYMBOL;
	const InstAnalysis *ia = qbdi_getInstAnalysis(vm, flag);

	if (ia->symbolName != NULL) {
		const char *sym  = ia->symbolName;
		const char *kind = classify(ia);
		increment(sym, kind);
	}

	return QBDI_CONTINUE;
}

int qbdipreload_on_run(VMInstanceRef vm, rword start, rword stop) {
	qbdi_addCodeCB(vm, QBDI_PREINST, onInstruction, NULL, 0);
	qbdi_run(vm, start, stop);
	return QBDIPRELOAD_NO_ERROR;
}

int qbdipreload_on_exit(int status) { 
	for (int i = 0; i < HASH_SIZE; i++) {
		Entry *e = table[i];
		while (e) {
			printf("bm::%s/%s/%" PRIu64 "\n", e->symbol, e->kind, e->count);
			e = e->next;
		}
	}

	return QBDIPRELOAD_NO_ERROR;
}
