#include "wrapper.h"

int vips_rs_set_property( VipsObject *object, const char *name, const GValue *value )
{
    VipsObjectClass *object_class = VIPS_OBJECT_GET_CLASS( object );
    GType type = G_VALUE_TYPE( value );

    GParamSpec *pspec;
    VipsArgumentClass *argument_class;
    VipsArgumentInstance *argument_instance;

    if( vips_object_get_argument( object, name,
                &pspec, &argument_class, &argument_instance ) ) {
        return -1;
    }

    if( G_IS_PARAM_SPEC_ENUM( pspec ) &&
            type == G_TYPE_STRING ) {
        GType pspec_type = G_PARAM_SPEC_VALUE_TYPE( pspec );

        int enum_value;
        GValue value2 = { 0 };

        if( (enum_value = vips_enum_from_nick( object_class->nickname,
                        pspec_type, g_value_get_string( value ) )) < 0 ) {
            return -1;
        }

        g_value_init( &value2, pspec_type );
        g_value_set_enum( &value2, enum_value );
        g_object_set_property( G_OBJECT( object ), name, &value2 );
        g_value_unset( &value2 );
    }
    else {
        g_object_set_property( G_OBJECT( object ), name, value );
    }

    return 1;
}

GType vips_rs_gint_get_type() {
    return G_TYPE_INT;
}

GType vips_rs_gdouble_get_type() {
    return G_TYPE_DOUBLE;
}

GType vips_rs_gboolean_get_type() {
    return G_TYPE_BOOLEAN;
}

GType vips_rs_gstring_get_type() {
    return G_TYPE_STRING;
}

GType vips_rs_gobject_get_type() {
    return G_TYPE_OBJECT;
}

GType vips_rs_get_type(const GValue *v) {
    return G_VALUE_TYPE(v);
}
