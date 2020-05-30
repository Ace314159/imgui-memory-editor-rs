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