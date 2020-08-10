import { StyleSheet } from 'react-native';

export const styles = StyleSheet.create({
  terms: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  arbitration: {
    borderWidth: 1,
  },
  arbiter: {
    backgroundColor: 'lightslategrey',
    padding: 10,
    borderWidth: 1,
  },
  player: {
    padding: 10,
    minWidth: 140,
    backgroundColor: 'lightslategrey',
  },
  payoutScript: {
    flex: 1,
    width: 150, 
    height: 150, 
    borderWidth: 2, 
    padding: 5,
  },
  pot: {
    flex: 1,
    padding: 5,
    margin: 30,
    alignItems: 'center',
    justifyContent: 'center',
  },
  arbitration: {
    margin: 10,
  },
  smallEmote: {
    width: 28,
    height: 28,
  },
  mediumEmote: {
    width: 56,
    height: 56,
  },
  arbiterImage: {
    width: 39,
    height: 27,
  },
  playerSelect: {
    flex: 1,
    backgroundColor: 'grey',
    alignItems: 'center',
    justifyContent: 'center',
  },
  playerSelector: {
    padding: 10,
    margin: 10,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
  },
  home: {
    flex: 1,
    backgroundColor: 'grey',
    alignItems: 'center',
    justifyContent: 'flex-start',
  },
  container: {
    flex: 1,
    backgroundColor: 'grey',
    alignItems: 'center',
    justifyContent: 'center',
  }
});
