#if defined(_MSC_VER) && !defined(_CRT_SECURE_NO_WARNINGS)
#define _CRT_SECURE_NO_WARNINGS

#ifdef _MSC_VER
#define _PRISizeT   "I"
#define ImSnprintf  _snprintf
#else
#define _PRISizeT   "z"
#define ImSnprintf  snprintf
#endif

void MemoryEditor::DrawWindow(const char* title, void* mem_data, size_t mem_size, size_t base_display_addr = 0x0000)
{
    Sizes s;
    CalcSizes(s, mem_size, base_display_addr);
    ImGui::SetNextWindowSizeConstraints(ImVec2(0.0f, 0.0f), ImVec2(s.WindowWidth, FLT_MAX));

    Open = true;
    if (ImGui::Begin(title, &Open, ImGuiWindowFlags_NoScrollbar))
    {
        if (ImGui::IsWindowHovered(ImGuiHoveredFlags_RootAndChildWindows) && ImGui::IsMouseClicked(1))
            ImGui::OpenPopup("context");
        DrawContents(mem_data, mem_size, base_display_addr);
        if (ContentsWidthChanged)
        {
            CalcSizes(s, mem_size, base_display_addr);
            ImGui::SetWindowSize(ImVec2(s.WindowWidth, ImGui::GetWindowSize().y));
        }
    }
    ImGui::End();
}

// Memory Editor contents only
void MemoryEditor::DrawContents(void* mem_data_void, size_t mem_size, size_t base_display_addr = 0x0000)
{
    if (Cols < 1)
        Cols = 1;

    ImU8* mem_data = (ImU8*)mem_data_void;
    Sizes s;
    CalcSizes(s, mem_size, base_display_addr);
    ImGuiStyle& style = ImGui::GetStyle();

    // We begin into our scrolling region with the 'ImGuiWindowFlags_NoMove' in order to prevent click from moving the window.
    // This is used as a facility since our main click detection code doesn't assign an ActiveId so the click would normally be caught as a window-move.
    const float height_separator = style.ItemSpacing.y;
    float footer_height = 0;
    if (OptShowOptions)
        footer_height += height_separator + ImGui::GetFrameHeightWithSpacing() * 1;
    if (OptShowDataPreview)
        footer_height += height_separator + ImGui::GetFrameHeightWithSpacing() * 1 + ImGui::GetTextLineHeightWithSpacing() * 3;
    ImGui::BeginChild("##scrolling", ImVec2(0, -footer_height), false, ImGuiWindowFlags_NoMove);
    ImDrawList* draw_list = ImGui::GetWindowDrawList();

    ImGui::PushStyleVar(ImGuiStyleVar_FramePadding, ImVec2(0, 0));
    ImGui::PushStyleVar(ImGuiStyleVar_ItemSpacing, ImVec2(0, 0));

    const int line_total_count = (int)((mem_size + Cols - 1) / Cols);
    ImGuiListClipper clipper(line_total_count, s.LineHeight);
    const size_t visible_start_addr = clipper.DisplayStart * Cols;
    const size_t visible_end_addr = clipper.DisplayEnd * Cols;

    bool data_next = false;

    if (ReadOnly || DataEditingAddr >= mem_size)
        DataEditingAddr = (size_t)-1;
    if (DataPreviewAddr >= mem_size)
        DataPreviewAddr = (size_t)-1;

    size_t preview_data_type_size = OptShowDataPreview ? DataTypeGetSize(PreviewDataType) : 0;

    size_t data_editing_addr_backup = DataEditingAddr;
    size_t data_editing_addr_next = (size_t)-1;
    if (DataEditingAddr != (size_t)-1)
    {
        // Move cursor but only apply on next frame so scrolling with be synchronized (because currently we can't change the scrolling while the window is being rendered)
        if (ImGui::IsKeyPressed(ImGui::GetKeyIndex(ImGuiKey_UpArrow)) && DataEditingAddr >= (size_t)Cols)          { data_editing_addr_next = DataEditingAddr - Cols; DataEditingTakeFocus = true; }
        else if (ImGui::IsKeyPressed(ImGui::GetKeyIndex(ImGuiKey_DownArrow)) && DataEditingAddr < mem_size - Cols) { data_editing_addr_next = DataEditingAddr + Cols; DataEditingTakeFocus = true; }
        else if (ImGui::IsKeyPressed(ImGui::GetKeyIndex(ImGuiKey_LeftArrow)) && DataEditingAddr > 0)               { data_editing_addr_next = DataEditingAddr - 1; DataEditingTakeFocus = true; }
        else if (ImGui::IsKeyPressed(ImGui::GetKeyIndex(ImGuiKey_RightArrow)) && DataEditingAddr < mem_size - 1)   { data_editing_addr_next = DataEditingAddr + 1; DataEditingTakeFocus = true; }
    }
    if (data_editing_addr_next != (size_t)-1 && (data_editing_addr_next / Cols) != (data_editing_addr_backup / Cols))
    {
        // Track cursor movements
        const int scroll_offset = ((int)(data_editing_addr_next / Cols) - (int)(data_editing_addr_backup / Cols));
        const bool scroll_desired = (scroll_offset < 0 && data_editing_addr_next < visible_start_addr + Cols * 2) || (scroll_offset > 0 && data_editing_addr_next > visible_end_addr - Cols * 2);
        if (scroll_desired)
            ImGui::SetScrollY(ImGui::GetScrollY() + scroll_offset * s.LineHeight);
    }

    // Draw vertical separator
    ImVec2 window_pos = ImGui::GetWindowPos();
    if (OptShowAscii)
        draw_list->AddLine(ImVec2(window_pos.x + s.PosAsciiStart - s.GlyphWidth, window_pos.y), ImVec2(window_pos.x + s.PosAsciiStart - s.GlyphWidth, window_pos.y + 9999), ImGui::GetColorU32(ImGuiCol_Border));

    const ImU32 color_text = ImGui::GetColorU32(ImGuiCol_Text);
    const ImU32 color_disabled = OptGreyOutZeroes ? ImGui::GetColorU32(ImGuiCol_TextDisabled) : color_text;

    const char* format_address = OptUpperCaseHex ? "%0*" _PRISizeT "X: " : "%0*" _PRISizeT "x: ";
    const char* format_data = OptUpperCaseHex ? "%0*" _PRISizeT "X" : "%0*" _PRISizeT "x";
    const char* format_byte = OptUpperCaseHex ? "%02X" : "%02x";
    const char* format_byte_space = OptUpperCaseHex ? "%02X " : "%02x ";

    for (int line_i = clipper.DisplayStart; line_i < clipper.DisplayEnd; line_i++) // display only visible lines
    {
        size_t addr = (size_t)(line_i * Cols);
        ImGui::Text(format_address, s.AddrDigitsCount, base_display_addr + addr);

        // Draw Hexadecimal
        for (int n = 0; n < Cols && addr < mem_size; n++, addr++)
        {
            float byte_pos_x = s.PosHexStart + s.HexCellWidth * n;
            if (OptMidColsCount > 0)
                byte_pos_x += (float)(n / OptMidColsCount) * s.SpacingBetweenMidCols;
            ImGui::SameLine(byte_pos_x);

            // Draw highlight
            bool is_highlight_from_user_range = (addr >= HighlightMin && addr < HighlightMax);
            bool is_highlight_from_user_func = (HighlightFn && HighlightFn(mem_data, addr));
            bool is_highlight_from_preview = (addr >= DataPreviewAddr && addr < DataPreviewAddr + preview_data_type_size);
            if (is_highlight_from_user_range || is_highlight_from_user_func || is_highlight_from_preview)
            {
                ImVec2 pos = ImGui::GetCursorScreenPos();
                float highlight_width = s.GlyphWidth * 2;
                bool is_next_byte_highlighted =  (addr + 1 < mem_size) && ((HighlightMax != (size_t)-1 && addr + 1 < HighlightMax) || (HighlightFn && HighlightFn(mem_data, addr + 1)));
                if (is_next_byte_highlighted || (n + 1 == Cols))
                {
                    highlight_width = s.HexCellWidth;
                    if (OptMidColsCount > 0 && n > 0 && (n + 1) < Cols && ((n + 1) % OptMidColsCount) == 0)
                        highlight_width += s.SpacingBetweenMidCols;
                }
                draw_list->AddRectFilled(pos, ImVec2(pos.x + highlight_width, pos.y + s.LineHeight), HighlightColor);
            }

            if (DataEditingAddr == addr)
            {
                // Display text input on current byte
                bool data_write = false;
                ImGui::PushID((void*)addr);
                if (DataEditingTakeFocus)
                {
                    ImGui::SetKeyboardFocusHere();
                    ImGui::CaptureKeyboardFromApp(true);
                    sprintf(AddrInputBuf, format_data, s.AddrDigitsCount, base_display_addr + addr);
                    sprintf(DataInputBuf, format_byte, ReadFn ? ReadFn(mem_data, addr) : mem_data[addr]);
                }
                ImGui::PushItemWidth(s.GlyphWidth * 2);
                struct UserData
                {
                    // FIXME: We should have a way to retrieve the text edit cursor position more easily in the API, this is rather tedious. This is such a ugly mess we may be better off not using InputText() at all here.
                    static int Callback(ImGuiInputTextCallbackData* data)
                    {
                        UserData* user_data = (UserData*)data->UserData;
                        if (!data->HasSelection())
                            user_data->CursorPos = data->CursorPos;
                        if (data->SelectionStart == 0 && data->SelectionEnd == data->BufTextLen)
                        {
                            // When not editing a byte, always rewrite its content (this is a bit tricky, since InputText technically "owns" the master copy of the buffer we edit it in there)
                            data->DeleteChars(0, data->BufTextLen);
                            data->InsertChars(0, user_data->CurrentBufOverwrite);
                            data->SelectionStart = 0;
                            data->SelectionEnd = data->CursorPos = 2;
                        }
                        return 0;
                    }
                    char   CurrentBufOverwrite[3];  // Input
                    int    CursorPos;               // Output
                };
                UserData user_data;
                user_data.CursorPos = -1;
                sprintf(user_data.CurrentBufOverwrite, format_byte, ReadFn ? ReadFn(mem_data, addr) : mem_data[addr]);
                ImGuiInputTextFlags flags = ImGuiInputTextFlags_CharsHexadecimal | ImGuiInputTextFlags_EnterReturnsTrue | ImGuiInputTextFlags_AutoSelectAll | ImGuiInputTextFlags_NoHorizontalScroll | ImGuiInputTextFlags_AlwaysInsertMode | ImGuiInputTextFlags_CallbackAlways;
                if (ImGui::InputText("##data", DataInputBuf, 32, flags, UserData::Callback, &user_data))
                    data_write = data_next = true;
                else if (!DataEditingTakeFocus && !ImGui::IsItemActive())
                    DataEditingAddr = data_editing_addr_next = (size_t)-1;
                DataEditingTakeFocus = false;
                ImGui::PopItemWidth();
                if (user_data.CursorPos >= 2)
                    data_write = data_next = true;
                if (data_editing_addr_next != (size_t)-1)
                    data_write = data_next = false;
                unsigned int data_input_value = 0;
                if (data_write && sscanf(DataInputBuf, "%X", &data_input_value) == 1)
                {
                    if (WriteFn)
                        WriteFn(mem_data, addr, (ImU8)data_input_value);
                    else
                        mem_data[addr] = (ImU8)data_input_value;
                }
                ImGui::PopID();
            }
            else
            {
                // NB: The trailing space is not visible but ensure there's no gap that the mouse cannot click on.
                ImU8 b = ReadFn ? ReadFn(mem_data, addr) : mem_data[addr];

                if (OptShowHexII)
                {
                    if ((b >= 32 && b < 128))
                        ImGui::Text(".%c ", b);
                    else if (b == 0xFF && OptGreyOutZeroes)
                        ImGui::TextDisabled("## ");
                    else if (b == 0x00)
                        ImGui::Text("   ");
                    else
                        ImGui::Text(format_byte_space, b);
                }
                else
                {
                    if (b == 0 && OptGreyOutZeroes)
                        ImGui::TextDisabled("00 ");
                    else
                        ImGui::Text(format_byte_space, b);
                }
                if (!ReadOnly && ImGui::IsItemHovered() && ImGui::IsMouseClicked(0))
                {
                    DataEditingTakeFocus = true;
                    data_editing_addr_next = addr;
                }
            }
        }

        if (OptShowAscii)
        {
            // Draw ASCII values
            ImGui::SameLine(s.PosAsciiStart);
            ImVec2 pos = ImGui::GetCursorScreenPos();
            addr = line_i * Cols;
            ImGui::PushID(line_i);
            if (ImGui::InvisibleButton("ascii", ImVec2(s.PosAsciiEnd - s.PosAsciiStart, s.LineHeight)))
            {
                DataEditingAddr = DataPreviewAddr = addr + (size_t)((ImGui::GetIO().MousePos.x - pos.x) / s.GlyphWidth);
                DataEditingTakeFocus = true;
            }
            ImGui::PopID();
            for (int n = 0; n < Cols && addr < mem_size; n++, addr++)
            {
                if (addr == DataEditingAddr)
                {
                    draw_list->AddRectFilled(pos, ImVec2(pos.x + s.GlyphWidth, pos.y + s.LineHeight), ImGui::GetColorU32(ImGuiCol_FrameBg));
                    draw_list->AddRectFilled(pos, ImVec2(pos.x + s.GlyphWidth, pos.y + s.LineHeight), ImGui::GetColorU32(ImGuiCol_TextSelectedBg));
                }
                unsigned char c = ReadFn ? ReadFn(mem_data, addr) : mem_data[addr];
                char display_c = (c < 32 || c >= 128) ? '.' : c;
                draw_list->AddText(pos, (display_c == '.') ? color_disabled : color_text, &display_c, &display_c + 1);
                pos.x += s.GlyphWidth;
            }
        }
    }
    clipper.End();
    ImGui::PopStyleVar(2);
    ImGui::EndChild();

    if (data_next && DataEditingAddr < mem_size)
    {
        DataEditingAddr = DataPreviewAddr = DataEditingAddr + 1;
        DataEditingTakeFocus = true;
    }
    else if (data_editing_addr_next != (size_t)-1)
    {
        DataEditingAddr = DataPreviewAddr = data_editing_addr_next;
    }

    const bool lock_show_data_preview = OptShowDataPreview;
    if (OptShowOptions)
    {
        ImGui::Separator();
        DrawOptionsLine(s, mem_data, mem_size, base_display_addr);
    }

    if (lock_show_data_preview)
    {
        ImGui::Separator();
        DrawPreviewLine(s, mem_data, mem_size, base_display_addr);
    }

    // Notify the main window of our ideal child content size (FIXME: we are missing an API to get the contents size from the child)
    ImGui::SetCursorPosX(s.WindowWidth);
}