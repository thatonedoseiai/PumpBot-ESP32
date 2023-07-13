//--Structs--
extern struct FT_Library;
extern struct FT_FaceRec_;
extern struct FT_Bitmap_;

//--Typedefs--
typedef unsigned char  FT_Bool;
typedef signed short  FT_FWord;
typedef unsigned short  FT_UFWord;
typedef signed char  FT_Char;
typedef unsigned char  FT_Byte;
typedef const FT_Byte*  FT_Bytes;
typedef FT_UInt32  FT_Tag;
typedef char  FT_String;
typedef signed short  FT_Short;
typedef unsigned short  FT_UShort;
typedef signed int  FT_Int;
typedef unsigned int  FT_UInt;
typedef signed long  FT_Long;
typedef unsigned long  FT_ULong;
typedef signed short  FT_F2Dot14;
typedef signed long  FT_F26Dot6;
typedef signed long  FT_Fixed;
typedef int  FT_Error;
typedef void*  FT_Pointer;
typedef size_t  FT_Offset;
typedef signed int FT_Int32;
typedef unsigned int FT_UInt32;

//--Functions--
extern void ft_logging_init(void);
extern int FT_Init_FreeType(FT_Library);
extern int FT_New_Face(FT_Library, const char*, FT_Long, FT_Face*);
extern int FT_Set_Char_Size(FT_Face, FT_F26Dot6, FT_F26Dot6, FT_UInt, FT_UInt);
extern int FT_Load_Char(FT_Face,FT_ULong, FT_Int32);