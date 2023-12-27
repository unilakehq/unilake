module.exports = {
    content: ["./**/*.{html,js,mjs,md,cshtml,razor,cs}", "./Pages/**/*.{cshtml,razor}"],
    darkMode: 'class',
    theme: {
        extend: {
            screens: {
                'sm': '640px',
                'md': '768px',
                'lg': '1024px',
                'xl': '1280px',
                '2xl': '1440px',
                '3xl': '1920px'
            },
            colors: {
                backgroundSecondary: '#121212',
                backgroundTertiary: '#0F0F0F',
                brandPrimary: '#F37441',
                textSecondary: '#CCCCCC',
                borderSecondary: '#383838',
                borderPrimary: '#6F6E73',
                textTertiary: '#6F6E73',
                textPrimary: '#E6EFF4',
                onBrandPrimary: '#4F1B05',
                brandSecondary: '#822C08',
                errorPrimary:'#F8D7D7',
                onErrorPrimary:'#571616'

            },
            boxShadow: {

            },
            fontSize: {
                Medium: ['16px', { lineHeight: '19.36px', fontWeight: '500' }],
                H5: ['18px', { lineHeight: '21.78px', fontWeight: '500' }],
                H4: ['24px', { lineHeight: '29.05px', fontWeight: '600' }],
                H3: ['32px', { lineHeight: '38.73px', fontWeight: '600' }],
                Regular: ['14px', { lineHeight: '16.94px', fontWeight: '400' }],

            },
            fontFamily: {
                inter: 'Inter , sans-serif'
            },
            width: {
                full: '100%',
                ArrowForward: '10.67px',
                Tag: '73px',
            },
            height: {
                Tag: '27px',
                ArrowForward: '10.67px'
            },
            top: {
                ArrowForward: '2.67px'
            },
            left: {
                ArrowForward: '2.67px'
            },
            borderWidth: {
                DEFAULT: '1px',
                Tag: '1px'
            },
            borderRadius: {
                DEFAULT: '0.25rem',
                XS: '4px',
                S: '8px',
                SM: '12px',
                M: '16px',
                L: '24px',
                XL: '32px',
                XXL: '100px',
            },
            padding: {
                XS: '4px',
                S: '8px',
                SM: '12px',
                M: '16px',
                L: '24px',
                XL: '32px',
                XXL: '100px'
            },
            spacing: {
                XS: '4px',
                S: '8px',
                SM: '12px',
                M: '16px',
                L: '24px',
                XL: '32px',
                XXL: '100px',
            }
        }
    },
    plugins: [
        require('flowbite/plugin')
    ]
}
