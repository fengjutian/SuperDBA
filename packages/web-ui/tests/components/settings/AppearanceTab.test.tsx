import {describe, it, expect, vi, beforeEach} from 'vitest';
import {render, screen, fireEvent} from '@testing-library/react';
import {MemoryRouter} from 'react-router-dom';
import React from 'react';

const updateSettings = vi.fn(async () => undefined);
const setTheme = vi.fn(async () => undefined);
let loading = false;

const lightTheme = {
    id: 'superdba-light',
    name: 'SuperDBA Light',
    monacoTheme: {base: 'vs'},
    colors: {
        accent: {primary: '#007acc', secondary: '#0098ff'},
        bg: {base: '#ffffff'},
        surface: {primary: '#f5f5f5'},
    },
};
const darkTheme = {
    id: 'superdba-dark',
    name: 'SuperDBA Dark',
    monacoTheme: {base: 'vs-dark'},
    colors: {
        accent: {primary: '#007acc', secondary: '#0098ff'},
        bg: {base: '#1a1a1a'},
        surface: {primary: '#2a2a2a'},
    },
};

let themeSettings = {
    activeThemeId: 'superdba-dark',
    followSystemTheme: false,
    lightThemeId: 'superdba-light',
    darkThemeId: 'superdba-dark',
    customThemes: [],
};

// The unified theme toolbar uses the same icons as its management menus.
vi.mock('@/hooks/useSuperDBAClient', () => import('../../support/tauriBackedHooks'));
vi.mock('@/hooks/usePlatformCapabilities', () => import('../../support/tauriBackedHooks'));
vi.mock('lucide-react', async () => await vi.importActual('lucide-react'));

vi.mock('react-i18next', () => ({
    useTranslation: () => ({t: (key: string) => key}),
}));

const updateSetting = vi.fn();

let settingsFontSize: number | undefined = 14;

vi.mock('@/hooks/useSettings', () => ({
    useSettings: () => ({
        settings: {fontFamily: 'System', fontSize: settingsFontSize},
        updateSetting,
    }),
}));

vi.mock('@/hooks/useTheme', () => ({
    useTheme: () => ({
        currentTheme: darkTheme,
        allThemes: [lightTheme, darkTheme],
        setTheme,
        settings: themeSettings,
        isLoading: loading,
        catalog: {themes: [], issues: []},
        updateSettings,
    }),
}));

vi.mock('@/components/settings/ResultColorsSection', () => ({
    ResultColorsSection: () => null,
}));

import {AppearanceTab} from '@/components/settings/AppearanceTab';

describe('AppearanceTab theme mode', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        themeSettings = {...themeSettings, followSystemTheme: false};
        settingsFontSize = 14;
        loading = false;
    });

    it('disables theme choices while preferences are hydrating', () => {
        loading = true;
        render(
            <MemoryRouter>
                <AppearanceTab />
            </MemoryRouter>,
        );
        expect(screen.getByText('SuperDBA Light').closest('button')).toBeDisabled();
    });

    it('surfaces preference save failures instead of an unhandled promise', async () => {
        updateSettings.mockRejectedValueOnce(new Error('settings unavailable'));
        render(
            <MemoryRouter>
                <AppearanceTab />
            </MemoryRouter>,
        );
        fireEvent.click(screen.getByText('settings.themeModeSystem'));
        expect(await screen.findByRole('alert')).toHaveTextContent('settings unavailable');
    });

    it('shows a single theme picker in static mode', () => {
        render(
            <MemoryRouter>
                <AppearanceTab />
            </MemoryRouter>,
        );
        expect(screen.getByText('SuperDBA Dark')).toBeTruthy();
        expect(screen.getByText('SuperDBA Light')).toBeTruthy();
        expect(screen.queryByText('settings.lightTheme')).toBeNull();
    });

    it('keeps theme management out of the independent SQL editor picker', () => {
        render(
            <MemoryRouter>
                <AppearanceTab />
            </MemoryRouter>,
        );
        fireEvent.click(screen.getByText('settings.appearance_sqlEditor'));
        expect(
            screen.getByRole('button', {name: 'settings.appearance_sameAsApp'}),
        ).toBeInTheDocument();
        expect(
            screen.queryByRole('region', {name: 'settings.themeSelection'}),
        ).not.toBeInTheDocument();
        expect(
            screen.queryByRole('button', {name: 'themePackages.import', exact: true}),
        ).not.toBeInTheDocument();
    });

    it('toggles follow-system via the mode button group', () => {
        render(
            <MemoryRouter>
                <AppearanceTab />
            </MemoryRouter>,
        );
        fireEvent.click(screen.getByText('settings.themeModeSystem'));
        expect(updateSettings).toHaveBeenCalledWith({followSystemTheme: true});
    });

    it('shows filtered light/dark pickers in follow-system mode', () => {
        themeSettings = {...themeSettings, followSystemTheme: true};
        render(
            <MemoryRouter>
                <AppearanceTab />
            </MemoryRouter>,
        );
        // Light picker: only light themes
        const lightSection = screen.getByText('settings.lightTheme').parentElement!;
        expect(lightSection.textContent).toContain('SuperDBA Light');
        expect(lightSection.textContent).not.toContain('SuperDBA Dark');
        // Dark picker: only dark themes
        const darkSection = screen.getByText('settings.darkTheme').parentElement!;
        expect(darkSection.textContent).toContain('SuperDBA Dark');
        expect(darkSection.textContent).not.toContain('SuperDBA Light');
    });

    it('updates lightThemeId when a light theme is picked', () => {
        themeSettings = {...themeSettings, followSystemTheme: true};
        render(
            <MemoryRouter>
                <AppearanceTab />
            </MemoryRouter>,
        );
        const lightSection = screen.getByText('settings.lightTheme').parentElement!;
        fireEvent.click(
            Array.from(lightSection.querySelectorAll('button')).find((b) =>
                b.textContent?.includes('SuperDBA Light'),
            )!,
        );
        expect(updateSettings).toHaveBeenCalledWith({
            lightThemeId: 'superdba-light',
        });
    });
});

describe('AppearanceTab base font size', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        settingsFontSize = 14;
        themeSettings = {...themeSettings, followSystemTheme: false};
        loading = false;
    });

    it('falls back to the 15px default when the setting is missing', () => {
        settingsFontSize = undefined;
        render(
            <MemoryRouter>
                <AppearanceTab />
            </MemoryRouter>,
        );
        expect(screen.getByText('15px')).toBeInTheDocument();
    });

    it('adjusts the base font size with the stepper buttons', () => {
        render(
            <MemoryRouter>
                <AppearanceTab />
            </MemoryRouter>,
        );
        fireEvent.click(screen.getByRole('button', {name: 'settings.fontSizeIncrease'}));
        expect(updateSetting).toHaveBeenCalledWith('fontSize', 15);
        fireEvent.click(screen.getByRole('button', {name: 'settings.fontSizeDecrease'}));
        expect(updateSetting).toHaveBeenCalledWith('fontSize', 13);
    });
});
