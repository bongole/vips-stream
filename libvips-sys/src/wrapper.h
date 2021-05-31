#include <vips/vips.h>
#include <vips/vector.h>

int vips_rs_set_property( VipsObject *object, const char *name, const GValue *value );
GType vips_rs_gint_get_type();
GType vips_rs_gdouble_get_type();
GType vips_rs_gboolean_get_type();
GType vips_rs_gstring_get_type();
GType vips_rs_get_type(const GValue *v);
