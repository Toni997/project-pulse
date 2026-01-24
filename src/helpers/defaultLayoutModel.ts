import { IJsonModel } from "flexlayout-react";
import Browser from "../components/Browser";
import Timeline from "../components/Timeline";

const defaultLayoutModel: IJsonModel = {
  global: {
    splitterSize: 3,
    splitterExtra: 7,
    tabEnablePopout: true,
    tabSetEnableClose: true,
    tabSetEnableTabWrap: true,
    tabSetMinWidth: 100,
    tabSetMinHeight: 100,
    borderMinSize: 100,
    borderEnableAutoHide: true,
    tabEnableDrag: true,
    tabSetEnableDrag: true,
    tabSetEnableDrop: true,
  },
  borders: [
    {
      type: 'border',
      selected: 0,
      size: 196,
      location: 'left',
      children: [
        {
          type: 'tab',
          id: '#0a7988f1-0cfb-4420-a2a3-8aa8af12684f',
          name: Browser.name,
          component: Browser.name,
          enableClose: false,
        },
      ],
    },
  ],
  layout: {
    type: 'row',
    id: '#baeaf63a-7e31-4f5b-836c-6aec43cb228a',
    children: [
      {
        type: 'row',
        id: '#2c81525f-f8e2-4e48-a94b-910ddbb5b91c',
        weight: 42.14932625516559,
        children: [
          {
            type: 'tabset',
            id: '#8b1bc920-89fb-4fe7-bad3-9cf70d693118',
            weight: 80,
            children: [
              {
                type: 'tab',
                id: '#8fcab2f4-23f8-49d7-ae82-f87a9c261200',
                name: Timeline.name,
                component: Timeline.name,
              },
            ],
            active: true,
          },
          {
            type: 'tabset',
            id: '#cde7aafe-4c76-444e-9c0a-2bdd4ed9c9b7',
            weight: 20,
            children: [
              {
                type: 'tab',
                id: '#d6a9c27a-701f-4b87-9661-be186dcb567c',
                name: 'Mixer',
                component: 'Mixer',
              },
            ],
          },
        ],
      },
    ],
  },
  popouts: {},
}

export default defaultLayoutModel;
