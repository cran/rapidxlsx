#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>

extern const char *rust_write_xlsx(SEXP path, SEXP sheet, SEXP df, SEXP comp);

SEXP c_write_xlsx(SEXP path, SEXP sheet, SEXP df, SEXP comp) {
    const char *err = rust_write_xlsx(path, sheet, df, comp);
    if (err != NULL) {
        Rf_error("%s", err);
    }
    return R_NilValue;
}

static const R_CallMethodDef CallEntries[] = {
    {"c_write_xlsx", (DL_FUNC) &c_write_xlsx, 4},
    {NULL, NULL, 0}
};

void R_init_rapidxlsx(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
}

__attribute__((visibility("hidden"))) void abort(void) {
    Rf_error("internal error in rapidxlsx");
}

