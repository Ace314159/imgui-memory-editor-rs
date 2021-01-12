#include "imgui.h"
#include "vendor/imgui-memory-editor/imgui_memory_editor/imgui_memory_editor.h"

extern "C" {
    void Editor_Create(MemoryEditor& editor) { editor = MemoryEditor(); }
    void Editor_DrawContents(MemoryEditor& editor, void* mem_data_void, size_t mem_size, size_t base_display_addr) {
        editor.DrawContents(mem_data_void, mem_size, base_display_addr);
    };
    void Editor_DrawWindow(MemoryEditor& editor, const char* title, void* mem_data, size_t mem_size, size_t base_display_addr) {
        editor.DrawWindow(title, mem_data, mem_size, base_display_addr);
    }
}
